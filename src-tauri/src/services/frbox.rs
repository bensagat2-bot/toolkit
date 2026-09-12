// FRBox (Aliyun PDS share) remote ZIP partition extraction for Transsion
// firmware. Port of the fucking-transsion frbox_client.rs + remote_zip.rs
// using the toolkit's blocking reqwest style.

use std::path::Path;

use flate2::read::DeflateDecoder;
use reqwest::blocking::Client;
use serde::Serialize;

const PDS_API: &str = "https://fra315.api.aliyunpds.com";

const EOCD_SIGNATURE: u32 = 0x06054b50;
const ZIP64_EOCD_LOCATOR_SIGNATURE: u32 = 0x07064b50;
const ZIP64_EOCD_SIGNATURE: u32 = 0x06064b50;
const CENTRAL_DIR_SIGNATURE: u32 = 0x02014b50;
const LOCAL_HEADER_SIGNATURE: u32 = 0x04034b50;

#[derive(Debug, Clone, Serialize)]
pub struct RemoteFile {
    pub filename: String,
    pub size: i64,
    pub size_human: String,
    pub file_id: String,
    pub path: String,
    pub isdir: bool,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RemoteZipEntry {
    pub name: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub local_header_offset: u64,
    pub compression_method: u16,
    pub crc32: u32,
}

fn client() -> Client {
    Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .timeout(std::time::Duration::from_secs(180))
        .build()
        .unwrap_or_default()
}

fn human_size(size: i64) -> String {
    let units = ["B", "KB", "MB", "GB", "TB"];
    let mut s = size as f64;
    for unit in &units {
        if s < 1024.0 {
            return format!("{:.1} {}", s, unit);
        }
        s /= 1024.0;
    }
    format!("{:.1} PB", s)
}

fn percent_decode(s: &str) -> String {
    let bytes = s.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&s[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    String::from_utf8_lossy(&out).to_string()
}

fn parse_url(url: &str) -> Result<(String, String), String> {
    let marker = "/disk/s/";
    let idx = url
        .find(marker)
        .ok_or_else(|| format!("Cannot parse share URL: {url}"))?;
    let rest = &url[idx + marker.len()..];
    let share_id: String = rest
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '-')
        .collect();
    if share_id.is_empty() {
        return Err(format!("Cannot parse share URL: {url}"));
    }
    let mut pwd = String::new();
    if let Some(qi) = rest.find('?') {
        for pair in rest[qi + 1..].split('&') {
            if let Some(eq) = pair.find('=') {
                if &pair[..eq] == "pwd" {
                    pwd = percent_decode(&pair[eq + 1..]);
                }
            }
        }
    }
    Ok((share_id, pwd))
}

fn get_share_token(c: &Client, share_id: &str, password: &str) -> Result<String, String> {
    let mut body = serde_json::json!({ "share_id": share_id });
    if !password.is_empty() {
        body["share_pwd"] = serde_json::json!(password);
    }
    let resp: serde_json::Value = c
        .post(format!("{PDS_API}/v2/share_link/get_share_token"))
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;
    if let Some(token) = resp.get("share_token").and_then(|t| t.as_str()) {
        return Ok(token.to_string());
    }
    let msg = resp
        .get("message")
        .and_then(|m| m.as_str())
        .or_else(|| resp.get("code").and_then(|c| c.as_str()))
        .unwrap_or("Unknown error");
    Err(format!("Failed to get share token: {msg}"))
}

fn list_files(c: &Client, token: &str, share_id: &str, parent_id: &str) -> Result<Vec<RemoteFile>, String> {
    let body = serde_json::json!({
        "share_id": share_id,
        "parent_file_id": parent_id,
        "limit": 100
    });
    let resp: serde_json::Value = c
        .post(format!("{PDS_API}/v2/file/list"))
        .header("x-share-token", token)
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;

    let items = match resp.get("items").and_then(|i| i.as_array()) {
        Some(arr) => arr,
        None => return Ok(Vec::new()),
    };

    let files: Vec<RemoteFile> = items
        .iter()
        .map(|item| {
            let size = item.get("size").and_then(|s| s.as_i64()).unwrap_or(0);
            RemoteFile {
                filename: item.get("name").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
                size,
                size_human: human_size(size),
                file_id: item.get("file_id").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                path: item.get("path").and_then(|v| v.as_str()).unwrap_or("").to_string(),
                isdir: item.get("type").and_then(|v| v.as_str()) == Some("folder"),
                mime_type: item.get("mime_type").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            }
        })
        .collect();
    Ok(files)
}

fn list_all_recursive(c: &Client, token: &str, share_id: &str, parent_id: &str, prefix: &str) -> Result<Vec<RemoteFile>, String> {
    let mut result = Vec::new();
    let files = list_files(c, token, share_id, parent_id)?;
    for mut f in files {
        f.path = if prefix.is_empty() {
            f.filename.clone()
        } else {
            format!("{prefix}/{}", f.filename)
        };
        if f.isdir {
            let sub_prefix = f.path.clone();
            let mut sub = list_all_recursive(c, token, share_id, &f.file_id, &sub_prefix)?;
            result.push(f);
            result.append(&mut sub);
        } else {
            result.push(f);
        }
    }
    Ok(result)
}

fn get_download_url(c: &Client, token: &str, share_id: &str, file_id: &str) -> Result<String, String> {
    let body = serde_json::json!({ "share_id": share_id, "file_id": file_id });
    let resp: serde_json::Value = c
        .post(format!("{PDS_API}/v2/file/get_download_url"))
        .header("x-share-token", token)
        .json(&body)
        .send()
        .map_err(|e| e.to_string())?
        .json()
        .map_err(|e| e.to_string())?;
    if let Some(url) = resp.get("url").and_then(|u| u.as_str()) {
        if !url.is_empty() {
            return Ok(url.to_string());
        }
    }
    let msg = resp
        .get("message")
        .and_then(|m| m.as_str())
        .unwrap_or("Unknown error");
    Err(format!("Failed to get download URL: {msg}"))
}

fn find_firmware_zip(files: &[RemoteFile]) -> Option<RemoteFile> {
    let mut zips: Vec<&RemoteFile> = files.iter().filter(|f| !f.isdir && f.filename.to_lowercase().ends_with(".zip")).collect();
    if zips.is_empty() {
        zips = files.iter().filter(|f| !f.isdir).collect();
    }
    zips.sort_by(|a, b| b.size.cmp(&a.size));
    zips.first().map(|f| f.clone())
}

// ---------- Remote ZIP parsing over HTTP Range ----------

fn read_u16(buf: &[u8], offset: usize) -> u16 {
    u16::from_le_bytes([buf[offset], buf[offset + 1]])
}

fn read_u32(buf: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3]])
}

fn read_u64(buf: &[u8], offset: usize) -> u64 {
    u64::from_le_bytes([
        buf[offset], buf[offset + 1], buf[offset + 2], buf[offset + 3],
        buf[offset + 4], buf[offset + 5], buf[offset + 6], buf[offset + 7],
    ])
}

fn find_signature_from_end(buf: &[u8], signature: u32) -> Option<usize> {
    let b0 = signature as u8;
    let b1 = (signature >> 8) as u8;
    let b2 = (signature >> 16) as u8;
    let b3 = (signature >> 24) as u8;
    (0..buf.len().saturating_sub(3)).rev().find(|&i| {
        buf[i] == b0 && buf[i + 1] == b1 && buf[i + 2] == b2 && buf[i + 3] == b3
    })
}

fn get_range(c: &Client, url: &str, start: u64, length: u64, token: Option<&str>) -> Result<Vec<u8>, String> {
    let range = format!("bytes={}-{}", start, start + length - 1);
    let mut req = c.get(url).header("Range", &range);
    if let Some(t) = token {
        req = req.header("x-share-token", t);
    }
    let resp = req.send().map_err(|e| e.to_string())?;
    if resp.status().as_u16() != 206 {
        return Err(format!("Server returned {} instead of 206 Partial Content", resp.status()));
    }
    resp.bytes().map(|b| b.to_vec()).map_err(|e| e.to_string())
}

fn get_suffix_range(c: &Client, url: &str, suffix_length: usize, token: Option<&str>) -> Result<Vec<u8>, String> {
    let range = format!("bytes=-{suffix_length}");
    let mut req = c.get(url).header("Range", &range);
    if let Some(t) = token {
        req = req.header("x-share-token", t);
    }
    let resp = req.send().map_err(|e| e.to_string())?;
    if resp.status().as_u16() != 206 {
        return Err(format!("Server returned {} instead of 206 Partial Content", resp.status()));
    }
    resp.bytes().map(|b| b.to_vec()).map_err(|e| e.to_string())
}

fn list_remote_zip_entries(c: &Client, url: &str, token: Option<&str>) -> Result<Vec<RemoteZipEntry>, String> {
    let tail_size = 22 + 65535 + 20;
    let tail = get_suffix_range(c, url, tail_size, token)?;
    let eocd_pos = find_signature_from_end(&tail, EOCD_SIGNATURE)
        .ok_or_else(|| "Could not find ZIP end-of-central-directory record".to_string())?;

    let mut cd_offset = read_u32(&tail, eocd_pos + 16) as u64;
    let mut cd_size = read_u32(&tail, eocd_pos + 12) as u64;
    let total_entries = read_u16(&tail, eocd_pos + 10) as u64;

    if cd_offset == 0xFFFFFFFF || cd_size == 0xFFFFFFFF || total_entries == 0xFFFF {
        let locator_pos = eocd_pos.checked_sub(20).ok_or_else(|| "Zip64 locator position underflow".to_string())?;
        if read_u32(&tail, locator_pos) != ZIP64_EOCD_LOCATOR_SIGNATURE {
            return Err("ZIP needs Zip64 but locator is missing".to_string());
        }
        let zip64_eocd_offset = read_u64(&tail, locator_pos + 8);
        let zip64_rec = get_range(c, url, zip64_eocd_offset, 56, token)?;
        if read_u32(&zip64_rec, 0) != ZIP64_EOCD_SIGNATURE {
            return Err("Zip64 EOCD record looks corrupt".to_string());
        }
        cd_size = read_u64(&zip64_rec, 40);
        cd_offset = read_u64(&zip64_rec, 48);
    }

    let cd = get_range(c, url, cd_offset, cd_size, token)?;
    let mut results = Vec::new();
    let mut pos = 0usize;
    while pos + 46 <= cd.len() {
        if read_u32(&cd, pos) != CENTRAL_DIR_SIGNATURE {
            break;
        }
        let comp_method = read_u16(&cd, pos + 10);
        let crc32 = read_u32(&cd, pos + 16);
        let mut comp_size = read_u32(&cd, pos + 20) as u64;
        let mut uncomp_size = read_u32(&cd, pos + 24) as u64;
        let name_len = read_u16(&cd, pos + 28) as usize;
        let extra_len = read_u16(&cd, pos + 30) as usize;
        let comment_len = read_u16(&cd, pos + 32) as usize;
        let mut local_header_offset = read_u32(&cd, pos + 42) as u64;

        let name = String::from_utf8_lossy(&cd[pos + 46..pos + 46 + name_len]).to_string();

        if comp_size == 0xFFFFFFFF || uncomp_size == 0xFFFFFFFF || local_header_offset == 0xFFFFFFFF {
            let extra_start = pos + 46 + name_len;
            let extra_end = extra_start + extra_len;
            let mut p = extra_start;
            while p + 4 <= extra_end {
                let id = read_u16(&cd, p);
                let size = read_u16(&cd, p + 2) as usize;
                let data_start = p + 4;
                if id == 0x0001 {
                    let mut c2 = 0;
                    if uncomp_size == 0xFFFFFFFF && data_start + c2 + 8 <= extra_end {
                        uncomp_size = read_u64(&cd, data_start + c2);
                        c2 += 8;
                    }
                    if comp_size == 0xFFFFFFFF && data_start + c2 + 8 <= extra_end {
                        comp_size = read_u64(&cd, data_start + c2);
                        c2 += 8;
                    }
                    if local_header_offset == 0xFFFFFFFF && data_start + c2 + 8 <= extra_end {
                        local_header_offset = read_u64(&cd, data_start + c2);
                    }
                    break;
                }
                p += 4 + size;
            }
        }

        results.push(RemoteZipEntry {
            name,
            compressed_size: comp_size,
            uncompressed_size: uncomp_size,
            local_header_offset,
            compression_method: comp_method,
            crc32,
        });
        pos += 46 + name_len + extra_len + comment_len;
    }
    Ok(results)
}

fn resolve_entry(c: &Client, url: &str, entry_name: &str, token: Option<&str>) -> Result<(RemoteZipEntry, u64), String> {
    let entries = list_remote_zip_entries(c, url, token)?;
    let entry = entries
        .iter()
        .find(|e| e.name == entry_name)
        .ok_or_else(|| format!("Entry not found in zip: {entry_name}"))?
        .clone();

    let header = get_range(c, url, entry.local_header_offset, 256, token)?;
    if read_u32(&header, 0) != LOCAL_HEADER_SIGNATURE {
        return Err(format!("Local file header for '{entry_name}' looks corrupt"));
    }
    let name_len = read_u16(&header, 26) as u64;
    let extra_len = read_u16(&header, 28) as u64;
    let data_start = entry.local_header_offset + 30 + name_len + extra_len;
    Ok((entry, data_start))
}

fn extract_entry(c: &Client, url: &str, entry: &RemoteZipEntry, data_start: u64, dest: &str, token: Option<&str>) -> Result<(), String> {
    let out = Path::new(dest);
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    if entry.compressed_size == 0 {
        std::fs::File::create(out).map_err(|e| e.to_string())?;
        return Ok(());
    }

    let range = format!("bytes={}-{}", data_start, data_start + entry.compressed_size - 1);
    let mut req = c.get(url).header("Range", range);
    if let Some(t) = token {
        req = req.header("x-share-token", t);
    }
    let resp = req.send().map_err(|e| e.to_string())?;
    if resp.status().as_u16() != 206 {
        return Err(format!("Server returned {} instead of 206 Partial Content", resp.status()));
    }

    let mut out_f = std::fs::File::create(out).map_err(|e| e.to_string())?;
    match entry.compression_method {
        0 => {
            let mut r = resp;
            std::io::copy(&mut r, &mut out_f).map_err(|e| e.to_string())?;
        }
        8 => {
            let mut r = DeflateDecoder::new(resp);
            std::io::copy(&mut r, &mut out_f).map_err(|e| e.to_string())?;
        }
        other => {
            let _ = std::fs::remove_file(out);
            return Err(format!("Unsupported compression method {other}"));
        }
    }
    Ok(())
}

// ---------- Public API (called from commands) ----------

pub fn list_partitions(url: &str, pwd: Option<&str>) -> Result<Vec<RemoteZipEntry>, String> {
    let (share_id, url_pwd) = parse_url(url)?;
    let password = pwd.unwrap_or(url_pwd.as_str());
    let c = client();

    let token = get_share_token(&c, &share_id, password)?;
    let files = list_all_recursive(&c, &token, &share_id, "root", "")?;
    let zip = find_firmware_zip(&files).ok_or_else(|| "No firmware archive found in share".to_string())?;
    let dl = get_download_url(&c, &token, &share_id, &zip.file_id)?;

    list_remote_zip_entries(&c, &dl, Some(&token))
}

pub fn extract_partition(url: &str, pwd: Option<&str>, name: &str, output_path: &str) -> Result<String, String> {
    let (share_id, url_pwd) = parse_url(url)?;
    let password = pwd.unwrap_or(url_pwd.as_str());
    let c = client();

    let token = get_share_token(&c, &share_id, password)?;
    let files = list_all_recursive(&c, &token, &share_id, "root", "")?;
    let zip = find_firmware_zip(&files).ok_or_else(|| "No firmware archive found in share".to_string())?;
    let dl = get_download_url(&c, &token, &share_id, &zip.file_id)?;

    let (entry, data_start) = resolve_entry(&c, &dl, name, Some(&token))?;
    extract_entry(&c, &dl, &entry, data_start, output_path, Some(&token))?;
    Ok(format!("{name} saved"))
}