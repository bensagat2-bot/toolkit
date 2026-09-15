use std::collections::HashMap;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;

use base64::Engine;
use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::services::utils;

fn rand_str(len: usize) -> String {
    let chars: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    let mut rng = rand::thread_rng();
    (0..len)
        .map(|_| chars[rng.gen_range(0..chars.len())] as char)
        .collect()
}

fn hmac_sha1_base64(key: &str, data: &str) -> String {
    let mut mac =
        hmac::Hmac::<sha1::Sha1>::new_from_slice(key.as_bytes()).expect("HMAC key");
    mac.update(data.as_bytes());
    let result = mac.finalize().into_bytes();
    base64::engine::general_purpose::STANDARD.encode(result)
}

fn xiaomi_sign(data: &serde_json::Value, ssecurity: &str, nonce: &str) -> HashMap<String, String> {
    let r = rand_str(16);
    let data_str = serde_json::to_string(data).unwrap_or_default();
    let mut params = vec![
        ("data".to_string(), data_str),
        ("r".to_string(), r.clone()),
        ("nonce".to_string(), nonce.to_string()),
    ];
    params.sort_by(|a, b| a.0.cmp(&b.0));
    let qs: String = params
        .iter()
        .map(|(k, v)| format!("{}={}", k, url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");
    let sign = hmac_sha1_base64(ssecurity, &qs);
    let mut result: HashMap<String, String> = params.into_iter().collect();
    result.insert("sign".to_string(), sign);
    result
}

fn url_encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char);
            }
            b' ' => out.push_str("%20"),
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoginResult {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionInfo {
    pub userId: String,
    pub ssecurity: String,
    pub deviceId: String,
    pub cookies: HashMap<String, String>,
    pub service_token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionInfo {
    pub region: String,
    pub zone: String,
    pub domain: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NonceResult {
    pub nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClearInfo {
    pub cleanOrNot: Option<i64>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockResult {
    pub encryptData: String,
    pub code: i64,
    pub description: Option<String>,
}

const ZONE_DOMAINS: &[(&str, &str)] = &[
    ("Singapore", "https://unlock.update.intl.miui.com"),
    ("China", "https://unlock.update.miui.com"),
    ("India", "https://in-unlock.update.intl.miui.com"),
    ("Russia", "https://ru-unlock.update.intl.miui.com"),
    ("Europe", "https://eu-unlock.update.intl.miui.com"),
];

const EU_REGIONS: &[&str] = &[
    "DE", "FR", "GB", "IT", "ES", "PL", "NL", "SE", "AT", "BE", "CH", "CZ", "DK", "FI", "GR",
    "HU", "IE", "NO", "PT", "RO", "SK", "UA",
];

fn map_region_to_zone(region: &str) -> &'static str {
    let r = region.to_uppercase();
    if r == "CN" || r == "CHINA" {
        "China"
    } else if r == "IN" || r == "INDIA" {
        "India"
    } else if r == "RU" || r == "RUSSIA" {
        "Russia"
    } else if EU_REGIONS.contains(&r.as_str()) {
        "Europe"
    } else {
        "Singapore"
    }
}

fn get_domain_for_zone(zone: &str) -> Option<&'static str> {
    ZONE_DOMAINS.iter().find(|(z, _)| *z == zone).map(|(_, d)| *d)
}

fn http_post_form(
    url: &str,
    params: &HashMap<String, String>,
    extra_cookies: Option<&HashMap<String, String>>,
) -> Result<serde_json::Value, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()
        .map_err(|e| e.to_string())?;
    let body: Vec<(String, String)> = params.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
    let mut req = client.post(url).form(&body);
    if let Some(cookies) = extra_cookies {
        let cookie_str: String = cookies
            .iter()
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("; ");
        req = req.header("Cookie", cookie_str);
    }
    let resp = req.send().map_err(|e| format!("HTTP error: {}", e))?;
    let status = resp.status();
    let text = resp.text().map_err(|e| format!("read error: {}", e))?;
    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, text));
    }
    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {} - body: {}", e, text))
}

fn http_get_with_cookies(
    url: &str,
    cookies: &HashMap<String, String>,
) -> Result<serde_json::Value, String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
        .build()
        .map_err(|e| e.to_string())?;
    let cookie_str: String = cookies
        .iter()
        .map(|(k, v)| format!("{}={}", k, v))
        .collect::<Vec<_>>()
        .join("; ");
    let resp = client
        .get(url)
        .header("Cookie", cookie_str)
        .send()
        .map_err(|e| format!("HTTP error: {}", e))?;
    let status = resp.status();
    let text = resp.text().map_err(|e| format!("read error: {}", e))?;
    if !status.is_success() {
        return Err(format!("HTTP {}: {}", status, text));
    }
    serde_json::from_str(&text).map_err(|e| format!("JSON parse error: {} - body: {}", e, text))
}

pub fn start_login() -> Result<LoginResult, String> {
    let listener =
        TcpListener::bind("127.0.0.1:0").map_err(|e| format!("Cannot bind port: {}", e))?;
    let port = listener.local_addr().map_err(|e| e.to_string())?.port();
    let state = rand_str(16);

    let redirect_uri = format!("http://127.0.0.1:{}/", port);
    let auth_url = format!(
        "https://account.xiaomi.com/oauth2/authorize?client_id=unlockApi&\
         redirect_uri={}&\
         response_type=code&\
         state={}&\
         skip_confirm=false",
        url_encode(&redirect_uri),
        state
    );

    open::that(&auth_url).map_err(|e| format!("Cannot open browser: {}", e))?;

    listener
        .set_nonblocking(true)
        .map_err(|e| format!("set_nonblocking: {}", e))?;

    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_secs(300);
    let mut code = String::new();
    let mut resp_state = String::new();
    let mut got_code = false;

    while start.elapsed() < timeout {
        if let Ok((mut stream, _)) = listener.accept() {
            let mut reader = BufReader::new(&mut stream);
            let mut request_line = String::new();
            if reader.read_line(&mut request_line).is_err() {
                continue;
            }
            let parts: Vec<&str> = request_line.split_whitespace().collect();
            if parts.len() >= 2 {
                let path = parts[1];
                if let Some(query) = path.split('?').nth(1) {
                    for pair in query.split('&') {
                        let kv: Vec<&str> = pair.splitn(2, '=').collect();
                        if kv.len() == 2 {
                            let key = url_decode(kv[1]);
                            if kv[0] == "code" {
                                code = key;
                            } else if kv[0] == "state" {
                                resp_state = key;
                            }
                        }
                    }
                }
            }
            let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n<!DOCTYPE html>\
                            <html><body style='font-family:sans-serif;text-align:center;\
                            padding:60px'><h2>✅ Login successful!</h2>\
                            <p>You can close this tab now.</p></body></html>";
            let _ = stream.write_all(response.as_bytes());
            got_code = true;
            break;
        }
        std::thread::sleep(std::time::Duration::from_millis(200));
    }

    if !got_code {
        return Err("Login timed out or was cancelled.".to_string());
    }
    if code.is_empty() {
        return Err("No authorization code received.".to_string());
    }
    if !resp_state.is_empty() && resp_state != state {
        return Err("State mismatch - possible CSRF.".to_string());
    }
    Ok(LoginResult {
        code,
        state: resp_state,
    })
}

fn url_decode(encoded: &str) -> String {
    let mut out = String::with_capacity(encoded.len());
    let mut chars = encoded.bytes();
    while let Some(b) = chars.next() {
        if b == b'%' {
            let hi = chars.next().and_then(|c| hex_val(c));
            let lo = chars.next().and_then(|c| hex_val(c));
            if let (Some(h), Some(l)) = (hi, lo) {
                out.push((h << 4 | l) as char);
            }
        } else if b == b'+' {
            out.push(' ');
        } else {
            out.push(b as char);
        }
    }
    out
}

fn hex_val(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

pub fn create_session(auth_code: &str) -> Result<SessionInfo, String> {
    let redirect_uri = "http://127.0.0.1:0/";

    let token_body: HashMap<String, String> = [
        ("client_id", "unlockApi"),
        ("code", auth_code),
        ("redirect_uri", redirect_uri),
        ("grant_type", "authorization_code"),
    ]
    .iter()
    .map(|(k, v)| (k.to_string(), v.to_string()))
    .collect();

    let token_resp =
        http_post_form("https://account.xiaomi.com/oauth2/token", &token_body, None)?;

    let pass_token = token_resp
        .get("access_token")
        .and_then(|v| v.as_str())
        .ok_or("No access_token in response")?;

    let location = format!("https://account.xiaomi.com/pass/service?sid=unlockApi");
    let mut cookies: HashMap<String, String> = HashMap::new();
    let cookie_pairs = [
        ("userId", token_resp.get("userId").and_then(|v| v.as_str())),
        ("serviceToken", Some(pass_token)),
    ];
    for (name, val) in cookie_pairs {
        if let Some(v) = val {
            cookies.insert(name.to_string(), v.to_string());
        }
    }
    let service_resp = http_get_with_cookies(&location, &cookies)?;
    let service_data = service_resp
        .get("data")
        .ok_or("No data in service response")?;
    let ssecurity = service_data
        .get("ssecurity")
        .and_then(|v| v.as_str())
        .ok_or("No ssecurity in service response")?
        .to_string();
    let device_id = service_data
        .get("deviceId")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let user_id = service_data
        .get("userId")
        .and_then(|v| v.as_str())
        .or_else(|| cookies.get("userId").map(|s| s.as_str()))
        .unwrap_or("")
        .to_string();

    Ok(SessionInfo {
        userId: user_id,
        ssecurity,
        deviceId: device_id,
        cookies,
        service_token: pass_token.to_string(),
    })
}

pub fn resolve_region(session: &SessionInfo) -> Result<RegionInfo, String> {
    let mut cookies = session.cookies.clone();
    cookies.insert(
        "serviceToken".to_string(),
        session.service_token.clone(),
    );
    let resp = http_get_with_cookies(
        "https://account.xiaomi.com/pass/region?sid=unlockApi",
        &cookies,
    )?;

    let region = resp
        .get("data")
        .and_then(|d| d.get("region"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let zone = map_region_to_zone(&region).to_string();
    let domain = get_domain_for_zone(&zone)
        .unwrap_or("https://unlock.update.intl.miui.com")
        .to_string();

    Ok(RegionInfo {
        region,
        zone,
        domain,
    })
}

pub fn xiaomi_send(
    path: &str,
    data: &serde_json::Value,
    domain: &str,
    ssecurity: &str,
    cookies: &HashMap<String, String>,
) -> Result<serde_json::Value, String> {
    let r = rand_str(16);
    let data_str = serde_json::to_string(data).map_err(|e| format!("JSON error: {}", e))?;
    let mut params: HashMap<String, String> = HashMap::new();
    params.insert("r".to_string(), r.clone());
    params.insert("data".to_string(), data_str);

    let mut sorted: Vec<(&String, &String)> = params.iter().collect();
    sorted.sort_by(|a, b| a.0.cmp(b.0));
    let qs: String = sorted
        .iter()
        .map(|(k, v)| format!("{}={}", k, url_encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let sign = hmac_sha1_base64(ssecurity, &qs);
    params.insert("sign".to_string(), sign);

    let url = format!("{}{}", domain.trim_end_matches('/'), path);
    http_post_form(&url, &params, Some(cookies))
}

pub fn get_nonce(
    domain: &str,
    ssecurity: &str,
    cookies: &HashMap<String, String>,
) -> Result<NonceResult, String> {
    let data = serde_json::json!({"r": rand_str(16)});
    let resp = xiaomi_send("/api/v2/nonce", &data, domain, ssecurity, cookies)?;
    let nonce = resp
        .get("nonce")
        .and_then(|v| v.as_str())
        .ok_or("No nonce in response")?
        .to_string();
    Ok(NonceResult { nonce })
}

pub fn get_clear_info(
    domain: &str,
    ssecurity: &str,
    cookies: &HashMap<String, String>,
    nonce: &str,
    product: &str,
) -> Result<ClearInfo, String> {
    let data = serde_json::json!({
        "appId": "1",
        "data": {
            "product": product,
        },
        "nonce": nonce,
    });
    let resp = xiaomi_send(
        "/api/v2/unlock/device/clear",
        &data,
        domain,
        ssecurity,
        cookies,
    )?;
    let clean_or_not = resp.get("data").and_then(|d| d.get("cleanOrNot")).and_then(|v| v.as_i64());
    let description = resp
        .get("data")
        .and_then(|d| d.get("description"))
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    Ok(ClearInfo {
        cleanOrNot: clean_or_not,
        description,
    })
}

pub fn get_device_token() -> Result<String, String> {
    let path = utils::fastboot_path();
    let out = utils::run_cmd(&path, &["oem", "get_token"], 15000);
    for line in out.lines() {
        let lower = line.to_lowercase();
        if let Some(idx) = lower.find("token:") {
            let val = line[idx + "token:".len()..].trim();
            if !val.is_empty() {
                return Ok(val.to_string());
            }
        }
    }
    Err("Could not read device token - is the device in fastboot mode?".to_string())
}

pub fn perform_unlock(
    session: &SessionInfo,
    region: &RegionInfo,
    product: &str,
    device_token: &str,
) -> Result<UnlockResult, String> {
    let domain = &region.domain;
    let ssecurity = &session.ssecurity;
    let cookies = &session.cookies;

    let nonce_resp = get_nonce(domain, ssecurity, cookies)?;
    let nonce = &nonce_resp.nonce;

    let data = serde_json::json!({
        "clientId": "2",
        "clientVersion": "7.6.727.43",
        "deviceInfo": {
            "boardVersion": "",
            "deviceName": "",
            "product": product,
            "socId": "",
        },
        "deviceToken": device_token,
        "language": "en",
        "operate": "unlock",
        "pcId": format!("{:x}", md5_hash(&session.deviceId)),
        "region": region.region,
        "uid": session.userId,
    });

    let req_data = serde_json::json!({
        "appId": "1",
        "data": data,
        "nonce": nonce,
    });

    let resp = xiaomi_send("/api/v3/ahaUnlock", &req_data, domain, ssecurity, cookies)?;

    let code = resp.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
    if code != 0 {
        let desc = resp
            .get("descEN")
            .and_then(|v| v.as_str())
            .or_else(|| resp.get("description").and_then(|v| v.as_str()))
            .unwrap_or("Unknown error");
        return Err(format!("Unlock API error ({}): {}", code, desc));
    }

    let encrypt_data = resp
        .get("encryptData")
        .and_then(|v| v.as_str())
        .ok_or("No encryptData in unlock response")?
        .to_string();

    apply_unlock(&encrypt_data)?;

    Ok(UnlockResult {
        encryptData: encrypt_data.clone(),
        code: 0,
        description: None,
    })
}

fn md5_hash(s: &str) -> String {
    let digest = md5::compute(s.as_bytes());
    format!("{:x}", digest)
}

fn apply_unlock(encrypt_data_hex: &str) -> Result<(), String> {
    let path = utils::fastboot_path();
    let tmp_dir = std::env::temp_dir();
    let token_file = tmp_dir.join(format!("{}_encryptData", std::time::UNIX_EPOCH.elapsed().unwrap_or_default().as_secs()));
    let data = hex::decode(encrypt_data_hex).map_err(|e| format!("Hex decode error: {}", e))?;
    std::fs::write(&token_file, &data).map_err(|e| format!("Write token file error: {}", e))?;

    let result = utils::run_cmd(&path, &["stage", &token_file.to_string_lossy()], 120000);
    if result.contains("FAILED") || result.contains("error") {
        let _ = std::fs::remove_file(&token_file);
        return Err(format!("fastboot stage failed: {}", result));
    }

    let result2 = utils::run_cmd(&path, &["oem", "unlock"], 60000);
    if result2.contains("FAILED") && !result2.contains("already") {
        let _ = std::fs::remove_file(&token_file);
        return Err(format!("fastboot oem unlock failed: {}", result2));
    }

    let _ = std::fs::remove_file(&token_file);
    Ok(())
}