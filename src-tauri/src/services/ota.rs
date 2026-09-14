use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek, SeekFrom, Write};
use std::sync::{mpsc, Arc, Mutex};

use flate2::read::GzDecoder;
use reqwest::blocking::Client;
use serde::Serialize;

use crate::config::OTA_MIRROR_HOST;

const BLOCK_SIZE: u64 = 4096;
const PAYLOAD_MEMBER: &str = "payload.bin";
const READAHEAD: u64 = 8 * 1024 * 1024;
// Parallel range workers + prefetch count (mirrors ota_extract.py).
const WORKERS: usize = 8;
const PREFETCH: u64 = 4;

pub fn mirror_url(url: &str) -> String {
    if url.contains("bigota.d.miui.com") || url.contains("ultimateota.d.miui.com") {
        if let Some(colon) = url.find("://") {
            let scheme = &url[..colon + 3];
            let rest = &url[colon + 3..];
            if let Some(slash) = rest.find('/') {
                return format!("{scheme}{OTA_MIRROR_HOST}{}", &rest[slash..]);
            }
        }
    }
    url.to_string()
}

fn client() -> Client {
    Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .connect_timeout(std::time::Duration::from_secs(10))
        .timeout(std::time::Duration::from_secs(300))
        .pool_max_idle_per_host(WORKERS + 4)
        .tcp_keepalive(std::time::Duration::from_secs(60))
        .build()
        .unwrap_or_default()
}

fn fetch_range(c: &Client, url: &str, start: u64, end: u64) -> Result<Vec<u8>, String> {
    let resp = c
        .get(url)
        .header("Range", format!("bytes={start}-{end}"))
        .send()
        .map_err(|e| format!("range request failed: {e}"))?;
    if resp.status().as_u16() != 206 && resp.status().as_u16() != 200 {
        return Err(format!("range request failed: HTTP {}", resp.status()));
    }
    resp.bytes()
        .map(|b| b.to_vec())
        .map_err(|e| format!("read failed: {e}"))
}

// ---------- Remote zip central directory parsing ----------

fn u16le(b: &[u8], off: usize) -> u16 {
    u16::from_le_bytes([b[off], b[off + 1]])
}

fn u32le(b: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([b[off], b[off + 1], b[off + 2], b[off + 3]])
}

fn u64le(b: &[u8], off: usize) -> u64 {
    u64::from_le_bytes([
        b[off],
        b[off + 1],
        b[off + 2],
        b[off + 3],
        b[off + 4],
        b[off + 5],
        b[off + 6],
        b[off + 7],
    ])
}

struct ZipEntry {
    name: String,
    local_header_offset: u64,
    size: u64,
}

/// Locate payload.bin inside a remote zip, returns (data_offset, size).
fn locate_payload(c: &Client, url: &str) -> Result<(u64, u64), String> {
    let resp = c
        .get(url)
        .header("Range", "bytes=0-0")
        .send()
        .map_err(|e| format!("head failed: {e}"))?;
    let total = resp
        .headers()
        .get("content-range")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.rsplit('/').next())
        .and_then(|v| v.trim().parse::<u64>().ok())
        .ok_or("cannot determine remote file size")?;

    // Include room for the ZIP64 EOCD locator (20 bytes before the EOCD).
    let tail_start = total.saturating_sub(22 + 65535 + 20);
    let tail = fetch_range(c, url, tail_start, total - 1)?;

    let mut eocd = None;
    let mut i = tail.len();
    while i >= 4 {
        i -= 1;
        if tail[i] == 0x50 && tail[i + 1] == 0x4b && tail[i + 2] == 0x05 && tail[i + 3] == 0x06 {
            eocd = Some(i);
            break;
        }
    }
    let eocd = eocd.ok_or("EOCD not found")?;

    let mut cd_size = u32le(&tail, eocd + 12) as u64;
    let mut cd_offset = u32le(&tail, eocd + 16) as u64;
    let total_entries = u16le(&tail, eocd + 10) as u64;

    // Xiaomi OTAs are larger than 4 GiB, so the 32-bit EOCD fields are the
    // ZIP64 sentinels and the real values live in the ZIP64 EOCD record.
    if cd_offset == 0xFFFF_FFFF || cd_size == 0xFFFF_FFFF || total_entries == 0xFFFF {
        let locator = eocd.checked_sub(20).ok_or("ZIP64 locator not found")?;
        if u32le(&tail, locator) != 0x0706_4b50 {
            return Err("ZIP64 locator missing".to_string());
        }
        let z64_off = u64le(&tail, locator + 8);
        let zrec = fetch_range(c, url, z64_off, z64_off + 55)?;
        if zrec.len() < 56 || u32le(&zrec, 0) != 0x0606_4b50 {
            return Err("ZIP64 EOCD record corrupt".to_string());
        }
        cd_size = u64le(&zrec, 40);
        cd_offset = u64le(&zrec, 48);
    }

    if cd_size > 64 * 1024 * 1024 {
        return Err("central directory too large".to_string());
    }

    let cd = fetch_range(c, url, cd_offset, cd_offset + cd_size - 1)?;
    let mut entries = Vec::new();
    let mut p = 0usize;
    while p + 46 <= cd.len() {
        if &cd[p..p + 4] != b"PK\x01\x02" {
            break;
        }
        let name_len = u16le(&cd, p + 28) as usize;
        let extra_len = u16le(&cd, p + 30) as usize;
        let comment_len = u16le(&cd, p + 32) as usize;
        let mut local_offset = u32le(&cd, p + 42) as u64;
        let mut size = u32le(&cd, p + 24) as u64;
        let compressed_is_z64 = u32le(&cd, p + 20) == 0xFFFF_FFFF;
        if p + 46 + name_len > cd.len() {
            break;
        }
        let name = String::from_utf8_lossy(&cd[p + 46..p + 46 + name_len]).to_string();

        // ZIP64 entries store the real values in the 0x0001 extra field, in a
        // fixed order (uncompressed size, compressed size, header offset, disk
        // start) but only for the 32-bit fields that hold the 0xFFFFFFFF sentinel.
        if size == 0xFFFF_FFFF || local_offset == 0xFFFF_FFFF {
            let extra = &cd[p + 46 + name_len..p + 46 + name_len + extra_len];
            let mut q = 0usize;
            while q + 4 <= extra.len() {
                let id = u16le(extra, q);
                let flen = u16le(extra, q + 2) as usize;
                let ds = q + 4;
                if id == 0x0001 {
                    let mut k = 0usize;
                    if size == 0xFFFF_FFFF && ds + k + 8 <= extra.len() {
                        size = u64le(extra, ds + k);
                        k += 8;
                    }
                    if compressed_is_z64 && ds + k + 8 <= extra.len() {
                        k += 8;
                    }
                    if local_offset == 0xFFFF_FFFF && ds + k + 8 <= extra.len() {
                        local_offset = u64le(extra, ds + k);
                    }
                    break;
                }
                q += 4 + flen;
            }
        }

        entries.push(ZipEntry {
            name,
            local_header_offset: local_offset,
            size,
        });
        p += 46 + name_len + extra_len + comment_len;
    }

    let entry = entries
        .iter()
        .find(|e| e.name == PAYLOAD_MEMBER)
        .ok_or("payload.bin not found in OTA")?;

    let lh = fetch_range(c, url, entry.local_header_offset, entry.local_header_offset + 29)?;
    if lh.len() < 30 || &lh[0..4] != b"PK\x03\x04" {
        return Err("invalid local header".to_string());
    }
    let name_len = u16le(&lh, 26) as u64;
    let extra_len = u16le(&lh, 28) as u64;
    let data_offset = entry.local_header_offset + 30 + name_len + extra_len;
    Ok((data_offset, entry.size))
}

// ---------- Minimal protobuf wire reader ----------

struct PbReader<'a> {
    buf: &'a [u8],
    pos: usize,
}

impl<'a> PbReader<'a> {
    fn new(buf: &'a [u8]) -> Self {
        Self { buf, pos: 0 }
    }
    fn varint(&mut self) -> Result<u64, String> {
        let mut result: u64 = 0;
        let mut shift = 0u32;
        loop {
            let b = *self.buf.get(self.pos).ok_or("varint overflow")?;
            self.pos += 1;
            result |= ((b & 0x7f) as u64) << shift;
            if b & 0x80 == 0 {
                return Ok(result);
            }
            shift += 7;
            if shift >= 64 {
                return Err("varint too long".to_string());
            }
        }
    }
    fn tag(&mut self) -> Result<(u64, u64), String> {
        let tag = self.varint()?;
        let field = tag >> 3;
        let wire = tag & 0x7;
        Ok((field, wire))
    }
    fn bytes(&mut self, len: usize) -> Result<&'a [u8], String> {
        if self.pos + len > self.buf.len() {
            return Err("field overrun".to_string());
        }
        let s = &self.buf[self.pos..self.pos + len];
        self.pos += len;
        Ok(s)
    }
}

// ---------- Payload manifest model ----------

#[derive(Clone, Serialize)]
pub struct PartitionInfo {
    pub name: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug)]
struct Extent {
    start_block: u64,
    num_blocks: u64,
}

#[derive(Clone, Debug)]
struct Operation {
    op_type: u32,
    data_offset: u64,
    data_length: u64,
    dst_extents: Vec<Extent>,
}

#[derive(Clone, Debug)]
struct Partition {
    name: String,
    operations: Vec<Operation>,
}

fn parse_manifest(manifest_raw: &[u8]) -> Result<Vec<Partition>, String> {
    let mut r = PbReader::new(manifest_raw);
    let mut partitions = Vec::new();
    loop {
        if r.pos >= manifest_raw.len() {
            break;
        }
        let (field, wire) = r.tag()?;
        match (field, wire) {
            (13, 2) => {
                let len = r.varint()? as usize;
                let data = r.bytes(len)?;
                partitions.push(parse_partition(data)?);
            }
            (_, 0) => {
                r.varint()?;
            }
            (_, 1) => {
                r.bytes(8)?;
            }
            (_, 2) => {
                let len = r.varint()? as usize;
                r.bytes(len)?;
            }
            (_, 5) => {
                r.bytes(4)?;
            }
            _ => return Err("unsupported wire type".to_string()),
        }
    }
    Ok(partitions)
}

fn parse_partition(data: &[u8]) -> Result<Partition, String> {
    let mut r = PbReader::new(data);
    let mut name = String::new();
    let mut operations = Vec::new();
    loop {
        if r.pos >= data.len() {
            break;
        }
        let (field, wire) = r.tag()?;
        match (field, wire) {
            (1, 2) => {
                let len = r.varint()? as usize;
                let s = r.bytes(len)?;
                name = String::from_utf8_lossy(s).to_string();
            }
            (8, 2) => {
                let len = r.varint()? as usize;
                let data = r.bytes(len)?;
                operations.push(parse_operation(data)?);
            }
            (_, 0) => {
                r.varint()?;
            }
            (_, 1) => {
                r.bytes(8)?;
            }
            (_, 2) => {
                let len = r.varint()? as usize;
                r.bytes(len)?;
            }
            (_, 5) => {
                r.bytes(4)?;
            }
            _ => return Err("unsupported wire type".to_string()),
        }
    }
    Ok(Partition { name, operations })
}

fn parse_operation(data: &[u8]) -> Result<Operation, String> {
    let mut r = PbReader::new(data);
    let mut op_type = 0u32;
    let mut data_offset = 0u64;
    let mut data_length = 0u64;
    let mut dst_extents = Vec::new();
    loop {
        if r.pos >= data.len() {
            break;
        }
        let (field, wire) = r.tag()?;
        match (field, wire) {
            (1, 0) => op_type = r.varint()? as u32,
            (2, 0) => data_offset = r.varint()?,
            (3, 0) => data_length = r.varint()?,
            (6, 2) => {
                let len = r.varint()? as usize;
                let data = r.bytes(len)?;
                let (start, blocks) = parse_extent(data)?;
                dst_extents.push(Extent {
                    start_block: start,
                    num_blocks: blocks,
                });
            }
            (_, 0) => {
                r.varint()?;
            }
            (_, 1) => {
                r.bytes(8)?;
            }
            (_, 2) => {
                let len = r.varint()? as usize;
                r.bytes(len)?;
            }
            (_, 5) => {
                r.bytes(4)?;
            }
            _ => return Err("unsupported wire type".to_string()),
        }
    }
    Ok(Operation {
        op_type,
        data_offset,
        data_length,
        dst_extents,
    })
}

fn parse_extent(data: &[u8]) -> Result<(u64, u64), String> {
    let mut r = PbReader::new(data);
    let mut start_block = 0u64;
    let mut num_blocks = 0u64;
    loop {
        if r.pos >= data.len() {
            break;
        }
        let (field, wire) = r.tag()?;
        match (field, wire) {
            (1, 0) => start_block = r.varint()?,
            (2, 0) => num_blocks = r.varint()?,
            (_, 0) => {
                r.varint()?;
            }
            (_, 1) => {
                r.bytes(8)?;
            }
            (_, 2) => {
                let len = r.varint()? as usize;
                r.bytes(len)?;
            }
            (_, 5) => {
                r.bytes(4)?;
            }
            _ => return Err("unsupported wire type".to_string()),
        }
    }
    Ok((start_block, num_blocks))
}

// ---------- Payload reader (seekable, range-backed, parallel) ----------

struct ReaderInner {
    cache: HashMap<u64, Arc<Vec<u8>>>,
    inflight: HashSet<u64>,
    errors: HashMap<u64, String>,
}

struct ReaderState {
    inner: Mutex<ReaderInner>,
    cond: std::sync::Condvar,
}

struct PayloadReader {
    size: u64,
    pos: u64,
    state: Arc<ReaderState>,
    job_tx: mpsc::Sender<u64>,
    _workers: Vec<std::thread::JoinHandle<()>>,
}

impl PayloadReader {
    fn new(client: Arc<Client>, url: String, base: u64, size: u64) -> Self {
        let (job_tx, job_rx) = mpsc::channel::<u64>();
        let job_rx = Arc::new(Mutex::new(job_rx));
        let state = Arc::new(ReaderState {
            inner: Mutex::new(ReaderInner {
                cache: HashMap::new(),
                inflight: HashSet::new(),
                errors: HashMap::new(),
            }),
            cond: std::sync::Condvar::new(),
        });

        let _workers: Vec<_> = (0..WORKERS)
            .map(|_| {
                let client = client.clone();
                let url = url.clone();
                let job_rx = job_rx.clone();
                let state = state.clone();
                std::thread::spawn(move || {
                    loop {
                        // Pop job WITHOUT holding the lock during HTTP fetch
                        let key = {
                            let Ok(rx) = job_rx.lock() else { break };
                            match rx.recv() {
                                Ok(k) => k,
                                Err(_) => break,
                            }
                        }; // Lock is dropped here immediately before network I/O

                        let start = key * READAHEAD;
                        let end = ((start + READAHEAD).min(size)) - 1;
                        let res = fetch_range(&client, &url, base + start, base + end);

                        {
                            let mut inner = state.inner.lock().unwrap();
                            match res {
                                Ok(data) => {
                                    inner.cache.insert(key, Arc::new(data));
                                }
                                Err(e) => {
                                    inner.errors.insert(key, e);
                                }
                            }
                            inner.inflight.remove(&key);
                            state.cond.notify_all();
                        }
                    }
                })
            })
            .collect();

        Self {
            size,
            pos: 0,
            state,
            job_tx,
            _workers,
        }
    }

    fn chunk(&self, key: u64) -> Result<Arc<Vec<u8>>, String> {
        let mut inner = self.state.inner.lock().unwrap();

        // 1. If already in cache or errored, return immediately!
        if let Some(d) = inner.cache.get(&key).cloned() {
            return Ok(d);
        }
        if let Some(e) = inner.errors.get(&key).cloned() {
            return Err(e);
        }

        // 2. Queue needed chunk FIRST, then queue prefetch chunks
        if !inner.inflight.contains(&key) {
            inner.inflight.insert(key);
            let _ = self.job_tx.send(key); // Target chunk goes first!

            for k in (key + 1)..(key + 1 + PREFETCH) {
                if k * READAHEAD >= self.size {
                    break;
                }
                if !inner.cache.contains_key(&k) && !inner.inflight.contains(&k) {
                    inner.inflight.insert(k);
                    let _ = self.job_tx.send(k);
                }
            }
        }

        // 3. Wait on condvar without busy-polling sleep
        let timeout = std::time::Duration::from_secs(120);
        let start = std::time::Instant::now();
        while !inner.cache.contains_key(&key) && !inner.errors.contains_key(&key) {
            let elapsed = start.elapsed();
            if elapsed >= timeout {
                return Err(format!("Timed out waiting for payload chunk {key}"));
            }
            let remaining = timeout - elapsed;
            let (new_inner, timeout_res) = self.state.cond.wait_timeout(inner, remaining).unwrap();
            inner = new_inner;
            if timeout_res.timed_out() && !inner.cache.contains_key(&key) && !inner.errors.contains_key(&key) {
                return Err(format!("Timed out waiting for payload chunk {key}"));
            }
        }

        if let Some(e) = inner.errors.get(&key).cloned() {
            return Err(e);
        }
        inner.cache.get(&key).cloned().ok_or_else(|| "Chunk missing".to_string())
    }

    fn read(&mut self, buf: &mut [u8]) -> Result<usize, String> {
        if self.pos >= self.size {
            return Ok(0);
        }
        let n = buf.len().min((self.size - self.pos) as usize);
        let mut written = 0usize;
        while written < n {
            let key = self.pos / READAHEAD;
            let off = (self.pos % READAHEAD) as usize;
            let data = self.chunk(key)?;
            let take = (n - written).min(data.len().saturating_sub(off));
            if take == 0 {
                self.pos = ((key + 1).saturating_mul(READAHEAD)).min(self.size);
                continue;
            }
            buf[written..written + take].copy_from_slice(&data[off..off + take]);
            self.pos += take as u64;
            written += take;
        }
        Ok(written)
    }
}

fn decompress_xz(data: &[u8], expected_size: u64) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut dec = xz2::read::XzDecoder::new(data);
    dec.read_to_end(&mut out)
        .map_err(|e| format!("xz decompress failed: {e}"))?;
    if out.len() as u64 != expected_size {
        return Err(format!(
            "xz size mismatch: got {} expected {expected_size}",
            out.len()
        ));
    }
    Ok(out)
}

fn decompress_bz2(data: &[u8], expected_size: u64) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut dec = bzip2::read::BzDecoder::new(data);
    dec.read_to_end(&mut out)
        .map_err(|e| format!("bz2 decompress failed: {e}"))?;
    if out.len() as u64 != expected_size {
        return Err(format!(
            "bz2 size mismatch: got {} expected {expected_size}",
            out.len()
        ));
    }
    Ok(out)
}

fn read_manifest(reader: &mut PayloadReader) -> Result<(Vec<Partition>, u64), String> {
    reader.pos = 0;
    let mut head = [0u8; 24];
    reader.read(&mut head)?;
    if &head[0..4] != b"CrAU" {
        return Err("invalid payload magic".to_string());
    }
    let version_arr: [u8; 8] = head[4..12].try_into().map_err(|_| "invalid payload version bytes")?;
    let manifest_len_arr: [u8; 8] = head[12..20].try_into().map_err(|_| "invalid manifest length bytes")?;
    let metadata_signature_len_arr: [u8; 4] = head[20..24].try_into().map_err(|_| "invalid metadata signature length bytes")?;
    let version = u64::from_be_bytes(version_arr);
    let manifest_len = u64::from_be_bytes(manifest_len_arr);
    let metadata_signature_len = u32::from_be_bytes(metadata_signature_len_arr);
    if version != 2 {
        return Err(format!("unsupported payload version ({version})"));
    }
    // Data blobs start right after the manifest + metadata signature.
    let data_offset = 24 + manifest_len + metadata_signature_len as u64;
    let mut manifest_raw = vec![0u8; manifest_len as usize];
    reader.read(&mut manifest_raw)?;
    parse_manifest(&manifest_raw).map(|parts| (parts, data_offset))
}

// ---------- Payload & Manifest Cache ----------

#[derive(Clone)]
struct CachedPayload {
    url: String,
    resolved_url: String,
    base: u64,
    size: u64,
    data_offset: u64,
    partitions: Vec<Partition>,
}

static CACHED_PAYLOAD: Mutex<Option<CachedPayload>> = Mutex::new(None);

fn get_or_open_payload(url: &str) -> Result<(Arc<Client>, String, u64, u64, u64, Vec<Partition>), String> {
    {
        let guard = CACHED_PAYLOAD.lock().unwrap();
        if let Some(cached) = guard.as_ref() {
            if cached.url == url {
                let c = client();
                return Ok((
                    Arc::new(c),
                    cached.resolved_url.clone(),
                    cached.base,
                    cached.size,
                    cached.data_offset,
                    cached.partitions.clone(),
                ));
            }
        }
    }

    let c = client();
    let c_arc = Arc::new(c.clone());
    let mut urls = Vec::new();
    let m = mirror_url(url);
    if m != url {
        urls.push(m);
    }
    urls.push(url.to_string());

    let mut last_err = String::from("no candidates");
    for u in &urls {
        match locate_payload(&c, u) {
            Ok((base, size)) => {
                let mut reader = PayloadReader::new(c_arc.clone(), u.clone(), base, size);
                match read_manifest(&mut reader) {
                    Ok((parts, data_offset)) => {
                        let mut guard = CACHED_PAYLOAD.lock().unwrap();
                        *guard = Some(CachedPayload {
                            url: url.to_string(),
                            resolved_url: u.clone(),
                            base,
                            size,
                            data_offset,
                            partitions: parts.clone(),
                        });
                        return Ok((c_arc, u.clone(), base, size, data_offset, parts));
                    }
                    Err(e) => last_err = e,
                }
            }
            Err(e) => last_err = e,
        }
    }
    Err(format!("cannot open OTA: {last_err}"))
}

// ---------- Public API ----------

pub fn list_partitions(url: &str) -> Result<Vec<PartitionInfo>, String> {
    let (_c, _u, _base, _size, _data_off, parts) = get_or_open_payload(url)?;
    let infos = parts
        .iter()
        .map(|p| {
            let bytes = p
                .operations
                .iter()
                .flat_map(|o| &o.dst_extents)
                .map(|e| e.num_blocks * BLOCK_SIZE)
                .sum();
            PartitionInfo {
                name: p.name.clone(),
                size_bytes: bytes,
            }
        })
        .collect();
    Ok(infos)
}

pub fn extract_partition(
    url: &str,
    partition_name: &str,
    output_path: &str,
) -> Result<String, String> {
    let (c_arc, u, base, size, data_offset, parts) = get_or_open_payload(url)?;
    let partition = parts
        .iter()
        .find(|p| p.name == partition_name)
        .ok_or_else(|| format!("Partition \"{partition_name}\" not found in OTA"))?;

    let mut reader = PayloadReader::new(c_arc, u, base, size);
    write_partition(&mut reader, partition, data_offset, output_path)?;
    Ok(format!("{partition_name}.img saved"))
}

fn write_partition(
    reader: &mut PayloadReader,
    partition: &Partition,
    data_offset: u64,
    output_path: &str,
) -> Result<(), String> {
    let out = std::path::Path::new(output_path);
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut out_f = std::fs::File::create(out).map_err(|e| e.to_string())?;

    for op in &partition.operations {
        let expected_size: u64 = op.dst_extents.iter().map(|e| e.num_blocks * BLOCK_SIZE).sum();
        match op.op_type {
            // REPLACE
            0 => {
                let data = read_op_data(reader, data_offset + op.data_offset, op.data_length)?;
                if data.len() as u64 != expected_size {
                    return Err(format!(
                        "REPLACE size mismatch: got {} expected {expected_size}",
                        data.len()
                    ));
                }
                write_extents(&mut out_f, &op.dst_extents, &data)?;
            }
            // REPLACE_BZ
            1 => {
                let data = read_op_data(reader, data_offset + op.data_offset, op.data_length)?;
                let decomp = decompress_bz2(&data, expected_size)?;
                write_extents(&mut out_f, &op.dst_extents, &decomp)?;
            }
            // ZERO
            6 => {
                for ext in &op.dst_extents {
                    let ext_bytes = ext.num_blocks * BLOCK_SIZE;
                    let dest_off = ext.start_block * BLOCK_SIZE;
                    out_f
                        .seek(SeekFrom::Start(dest_off))
                        .map_err(|e| e.to_string())?;
                    let zeros = [0u8; 4096];
                    let mut written = 0u64;
                    while written < ext_bytes {
                        let step = ((ext_bytes - written) as usize).min(zeros.len());
                        out_f
                            .write_all(&zeros[..step])
                            .map_err(|e| e.to_string())?;
                        written += step as u64;
                    }
                }
            }
            // REPLACE_XZ
            8 => {
                let data = read_op_data(reader, data_offset + op.data_offset, op.data_length)?;
                let decomp = decompress_xz(&data, expected_size)?;
                write_extents(&mut out_f, &op.dst_extents, &decomp)?;
            }
            other => {
                return Err(format!(
                    "unhandled operation type ({other}). Only REPLACE / REPLACE_BZ / REPLACE_XZ / ZERO supported"
                ));
            }
        }
    }
    Ok(())
}

fn write_extents(out_f: &mut std::fs::File, extents: &[Extent], data: &[u8]) -> Result<(), String> {
    let mut offset = 0usize;
    for ext in extents {
        let ext_bytes = (ext.num_blocks * BLOCK_SIZE) as usize;
        let dest_off = ext.start_block * BLOCK_SIZE;
        out_f
            .seek(SeekFrom::Start(dest_off))
            .map_err(|e| e.to_string())?;
        let end = (offset + ext_bytes).min(data.len());
        if offset < end {
            out_f
                .write_all(&data[offset..end])
                .map_err(|e| e.to_string())?;
            offset = end;
        }
    }
    Ok(())
}

fn read_op_data(reader: &mut PayloadReader, offset: u64, length: u64) -> Result<Vec<u8>, String> {
    reader.pos = offset;
    let mut buf = vec![0u8; length as usize];
    let mut done = 0usize;
    while done < buf.len() {
        let n = reader.read(&mut buf[done..])?;
        if n == 0 {
            return Err(format!("short read: got {done} of {length} bytes"));
        }
        done += n;
    }
    Ok(buf)
}

// ---------- Fastboot .tgz streaming extraction ----------

fn tar_parse_int(b: &[u8]) -> u64 {
    let b = b.split(|&x| x == 0).next().unwrap_or(b);
    let s = std::str::from_utf8(b).unwrap_or("0").trim();
    if s.is_empty() {
        return 0;
    }
    u64::from_str_radix(s, 8).unwrap_or(0)
}

pub fn extract_from_tgz(
    url: &str,
    image_name: &str,
    output_path: &str,
) -> Result<String, String> {
    let name = if image_name.ends_with(".img") {
        image_name.to_string()
    } else {
        format!("{image_name}.img")
    };

    let c = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
        .connect_timeout(std::time::Duration::from_secs(30))
        .timeout(std::time::Duration::from_secs(7200))
        .build()
        .map_err(|e| format!("client build failed: {e}"))?;
    let resp = c
        .get(url)
        .send()
        .map_err(|e| format!("download failed: {e}"))?;
    if !resp.status().is_success() {
        return Err(format!("download failed: HTTP {}", resp.status()));
    }

    let mut dec = GzDecoder::new(resp);
    let mut found = false;

    let out = std::path::Path::new(output_path);
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let mut out_f = std::fs::File::create(out).map_err(|e| e.to_string())?;

    loop {
        let mut header = [0u8; 512];
        if !read_exact(&mut dec, &mut header)? {
            break;
        }
        let h_name = header[0..100]
            .split(|&x| x == 0)
            .next()
            .unwrap_or(&header[0..100]);
        let h_name = String::from_utf8_lossy(h_name).to_string();
        let size = tar_parse_int(&header[124..136]);
        let typeflag = header[156];
        let padded = (size + 511) / 512 * 512;

        if typeflag != b'0' && typeflag != 0 {
            skip_bytes(&mut dec, padded)?;
            continue;
        }
        if h_name == name {
            let mut remaining = size;
            let mut chunk = vec![0u8; 1 << 16];
            while remaining > 0 {
                let need = remaining.min(chunk.len() as u64) as usize;
                let n = read_exact_chunk(&mut dec, &mut chunk[..need])?;
                if n == 0 {
                    break;
                }
                out_f.write_all(&chunk[..n]).map_err(|e| e.to_string())?;
                remaining -= n as u64;
            }
            skip_bytes(&mut dec, padded - size)?;
            found = true;
            break;
        } else {
            skip_bytes(&mut dec, padded)?;
        }
    }

    if !found {
        let _ = std::fs::remove_file(out);
        return Err(format!("Image \"{name}\" not found in fastboot archive"));
    }
    Ok(format!("{name} saved"))
}

fn read_exact(r: &mut impl std::io::Read, buf: &mut [u8]) -> Result<bool, String> {
    let mut done = 0usize;
    while done < buf.len() {
        let n = r.read(&mut buf[done..]).map_err(|e| e.to_string())?;
        if n == 0 {
            return Ok(done > 0);
        }
        done += n;
    }
    Ok(true)
}

fn read_exact_chunk(r: &mut impl std::io::Read, buf: &mut [u8]) -> Result<usize, String> {
    let mut done = 0usize;
    while done < buf.len() {
        let n = r.read(&mut buf[done..]).map_err(|e| e.to_string())?;
        if n == 0 {
            break;
        }
        done += n;
    }
    Ok(done)
}

fn skip_bytes(r: &mut impl std::io::Read, mut n: u64) -> Result<(), String> {
    let mut skip = vec![0u8; 1 << 16];
    while n > 0 {
        let step = n.min(skip.len() as u64) as usize;
        let read = read_exact_chunk(r, &mut skip[..step])?;
        if read == 0 {
            return Err("unexpected end of archive".to_string());
        }
        n -= read as u64;
    }
    Ok(())
}

// ---------- Fastboot .zip image listing (remote, no download) ----------

#[derive(Clone, Serialize)]
pub struct FastbootImageInfo {
    pub name: String,
    pub size_bytes: u64,
}

fn list_remote_zip_images(c: &Client, url: &str) -> Result<Vec<FastbootImageInfo>, String> {
    let resp = c.get(url).header("Range", "bytes=0-0").send().map_err(|e| format!("head failed: {e}"))?;
    let total = resp.headers().get("content-range")
        .and_then(|v| v.to_str().ok()).and_then(|v| v.rsplit('/').next())
        .and_then(|v| v.trim().parse::<u64>().ok()).ok_or("cannot determine remote file size")?;

    let tail_start = total.saturating_sub(22 + 65535 + 20);
    let tail = fetch_range(c, url, tail_start, total - 1)?;
    let mut eocd = None;
    let mut i = tail.len();
    while i >= 4 {
        i -= 1;
        if tail[i] == 0x50 && tail[i + 1] == 0x4b && tail[i + 2] == 0x05 && tail[i + 3] == 0x06 { eocd = Some(i); break; }
    }
    let eocd = eocd.ok_or("EOCD not found")?;
    let mut cd_size = u32le(&tail, eocd + 12) as u64;
    let mut cd_offset = u32le(&tail, eocd + 16) as u64;
    let total_entries = u16le(&tail, eocd + 10) as u64;

    if cd_offset == 0xFFFF_FFFF || cd_size == 0xFFFF_FFFF || total_entries == 0xFFFF {
        let locator = eocd.checked_sub(20).ok_or("ZIP64 locator not found")?;
        if u32le(&tail, locator) != 0x0706_4b50 { return Err("ZIP64 locator missing".to_string()); }
        let z64_off = u64le(&tail, locator + 8);
        let zrec = fetch_range(c, url, z64_off, z64_off + 55)?;
        if zrec.len() < 56 || u32le(&zrec, 0) != 0x0606_4b50 { return Err("ZIP64 EOCD record corrupt".to_string()); }
        cd_size = u64le(&zrec, 40);
        cd_offset = u64le(&zrec, 48);
    }
    if cd_size > 64 * 1024 * 1024 { return Err("central directory too large".to_string()); }

    let cd = fetch_range(c, url, cd_offset, cd_offset + cd_size - 1)?;
    let mut entries = Vec::new();
    let mut p = 0usize;
    while p + 46 <= cd.len() {
        if &cd[p..p + 4] != b"PK\x01\x02" { break; }
        let name_len = u16le(&cd, p + 28) as usize;
        let extra_len = u16le(&cd, p + 30) as usize;
        let comment_len = u16le(&cd, p + 32) as usize;
        let _local_offset = u32le(&cd, p + 42) as u64;
        let mut size = u32le(&cd, p + 24) as u64;
        let compressed_is_z64 = u32le(&cd, p + 20) == 0xFFFF_FFFF;
        if p + 46 + name_len > cd.len() { break; }
        let name = String::from_utf8_lossy(&cd[p + 46..p + 46 + name_len]).to_string();

        if size == 0xFFFF_FFFF {
            let extra = &cd[p + 46 + name_len..p + 46 + name_len + extra_len];
            let mut q = 0usize;
            while q + 4 <= extra.len() {
                let id = u16le(extra, q);
                let flen = u16le(extra, q + 2) as usize;
                let ds = q + 4;
                if id == 0x0001 {
                    let mut k = 0usize;
                    if size == 0xFFFF_FFFF && ds + k + 8 <= extra.len() { size = u64le(extra, ds + k); k += 8; }
                    // compressed_is_z64 field is present but we skip it (not needed for listing)
                    break;
                }
                q += 4 + flen;
            }
        }

        if name.to_lowercase().ends_with(".img") {
            let stem = name.rsplit('/').next().unwrap_or(&name).trim_end_matches(".img").to_string();
            entries.push(FastbootImageInfo { name: stem, size_bytes: size });
        }
        p += 46 + name_len + extra_len + comment_len;
    }
    entries.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(entries)
}

/// Lists all .img files in a remote fastboot .zip.
pub fn list_fastboot_images(url: &str) -> Result<Vec<FastbootImageInfo>, String> {
    let c = client();
    list_remote_zip_images(&c, url)
}

/// Extracts a single .img from a remote fastboot .zip using range requests.
pub fn extract_fastboot_image(url: &str, image_name: &str, output_path: &str) -> Result<String, String> {
    let name = if image_name.ends_with(".img") { image_name.to_string() } else { format!("{image_name}.img") };
    let c = client();

    let resp = c.get(url).header("Range", "bytes=0-0").send().map_err(|e| format!("head failed: {e}"))?;
    let total = resp.headers().get("content-range")
        .and_then(|v| v.to_str().ok()).and_then(|v| v.rsplit('/').next())
        .and_then(|v| v.trim().parse::<u64>().ok()).ok_or("cannot determine remote file size")?;

    let tail_start = total.saturating_sub(22 + 65535 + 20);
    let tail = fetch_range(&c, url, tail_start, total - 1)?;
    let mut eocd = None;
    let mut i = tail.len();
    while i >= 4 {
        i -= 1;
        if tail[i] == 0x50 && tail[i + 1] == 0x4b && tail[i + 2] == 0x05 && tail[i + 3] == 0x06 { eocd = Some(i); break; }
    }
    let eocd = eocd.ok_or("EOCD not found")?;

    let mut cd_size = u32le(&tail, eocd + 12) as u64;
    let mut cd_offset = u32le(&tail, eocd + 16) as u64;
    let total_entries = u16le(&tail, eocd + 10) as u64;
    if cd_offset == 0xFFFF_FFFF || cd_size == 0xFFFF_FFFF || total_entries == 0xFFFF {
        let locator = eocd.checked_sub(20).ok_or("ZIP64 locator not found")?;
        if u32le(&tail, locator) != 0x0706_4b50 { return Err("ZIP64 locator missing".to_string()); }
        let z64_off = u64le(&tail, locator + 8);
        let zrec = fetch_range(&c, url, z64_off, z64_off + 55)?;
        if zrec.len() < 56 || u32le(&zrec, 0) != 0x0606_4b50 { return Err("ZIP64 EOCD record corrupt".to_string()); }
        cd_size = u64le(&zrec, 40);
        cd_offset = u64le(&zrec, 48);
    }
    if cd_size > 64 * 1024 * 1024 { return Err("central directory too large".to_string()); }

    let cd = fetch_range(&c, url, cd_offset, cd_offset + cd_size - 1)?;
    let mut target_entry: Option<(u64, u64)> = None;
    let mut p = 0usize;
    while p + 46 <= cd.len() {
        if &cd[p..p + 4] != b"PK\x01\x02" { break; }
        let name_len = u16le(&cd, p + 28) as usize;
        let extra_len = u16le(&cd, p + 30) as usize;
        let comment_len = u16le(&cd, p + 32) as usize;
        let mut local_offset = u32le(&cd, p + 42) as u64;
        let mut size = u32le(&cd, p + 24) as u64;
        let compressed_is_z64 = u32le(&cd, p + 20) == 0xFFFF_FFFF;
        if p + 46 + name_len > cd.len() { break; }
        let raw_name = String::from_utf8_lossy(&cd[p + 46..p + 46 + name_len]).to_string();
        let file_name = raw_name.rsplit('/').next().unwrap_or(&raw_name).to_string();

        if size == 0xFFFF_FFFF || local_offset == 0xFFFF_FFFF {
            let extra = &cd[p + 46 + name_len..p + 46 + name_len + extra_len];
            let mut q = 0usize;
            while q + 4 <= extra.len() {
                let id = u16le(extra, q);
                let flen = u16le(extra, q + 2) as usize;
                let ds = q + 4;
                if id == 0x0001 {
                    let mut k = 0usize;
                    if size == 0xFFFF_FFFF && ds + k + 8 <= extra.len() { size = u64le(extra, ds + k); k += 8; }
                    if compressed_is_z64 && ds + k + 8 <= extra.len() { k += 8; }
                    if local_offset == 0xFFFF_FFFF && ds + k + 8 <= extra.len() { local_offset = u64le(extra, ds + k); }
                    break;
                }
                q += 4 + flen;
            }
        }

        let match_name = name.trim_end_matches(".img").to_lowercase();
        let raw_lower = file_name.trim_end_matches(".img").to_lowercase();
        if raw_lower == match_name || file_name == name || raw_name == name {
            target_entry = Some((local_offset, size));
            break;
        }
        p += 46 + name_len + extra_len + comment_len;
    }

    let (local_header_offset, size) = target_entry.ok_or_else(|| format!("Image \"{name}\" not found in fastboot archive"))?;

    let lh = fetch_range(&c, url, local_header_offset, local_header_offset + 29)?;
    if lh.len() < 30 || &lh[0..4] != b"PK\x03\x04" { return Err("invalid local header".to_string()); }
    let name_len = u16le(&lh, 26) as u64;
    let extra_len = u16le(&lh, 28) as u64;
    let data_offset = local_header_offset + 30 + name_len + extra_len;

    let out = std::path::Path::new(output_path);
    if let Some(parent) = out.parent() { std::fs::create_dir_all(parent).map_err(|e| e.to_string())?; }
    let mut out_f = std::fs::File::create(out).map_err(|e| e.to_string())?;

    let mut offset = data_offset;
    let mut remaining = size;
    let chunk_size = 4 * 1024 * 1024;
    while remaining > 0 {
        let end = (offset + remaining.min(chunk_size) - 1).min(offset + remaining - 1);
        let data = fetch_range(&c, url, offset, end)?;
        out_f.write_all(&data).map_err(|e| e.to_string())?;
        offset += data.len() as u64;
        remaining -= data.len() as u64;
    }

    Ok(format!("{name} saved"))
}