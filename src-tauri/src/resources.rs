use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

const ENC_PLATFORM_TOOLS: &[u8] = include_bytes!("../embed/platform-tools.zip.enc");
const ENC_UNISOC: &[u8] = include_bytes!("../embed/unisoc.zip.enc");

fn decipher() -> [u8; 12] {
    let mut k = [0u8; 12];
    k[0] = 0x56;
    k[1] = 0x31;
    k[2] = 0x50;
    k[3] = 0x45;
    k[4] = 0x72;
    k[5] = 0x53;
    k[6] = 0x65;
    k[7] = 0x72;
    k[8] = 0x76;
    k[9] = 0x49;
    k[10] = 0x63;
    k[11] = 0x65;
    k
}

fn decrypt(data: &[u8], key: &[u8]) -> Vec<u8> {
    let key_len = key.len();
    data.iter().enumerate().map(|(i, &b)| b ^ key[i % key_len]).collect()
}

fn cache_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("V1PerServicing")
        .join("cache")
}

fn extract_zip(data: &[u8], dest: &Path) -> Result<(), String> {
    if dest.exists() {
        return Ok(());
    }
    fs::create_dir_all(dest).map_err(|e| format!("Failed to create dir: {e}"))?;
    let cursor = std::io::Cursor::new(data);
    let mut archive = ZipArchive::new(cursor).map_err(|e| format!("Invalid zip: {e}"))?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).map_err(|e| format!("Zip read error: {e}"))?;
        let out_path = dest.join(file.name());
        if file.is_dir() {
            fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent).ok();
            }
            let mut out = fs::File::create(&out_path).map_err(|e| format!("Failed to create file: {e}"))?;
            let mut buf = Vec::new();
            file.read_to_end(&mut buf).map_err(|e| format!("Failed to read zip entry: {e}"))?;
            std::io::Write::write_all(&mut out, &buf).map_err(|e| format!("Failed to write file: {e}"))?;
        }
    }
    Ok(())
}

pub fn extract_all() -> Result<PathBuf, String> {
    let key = decipher();
    let root = cache_dir();
    fs::create_dir_all(&root).map_err(|e| format!("Failed to create cache dir: {e}"))?;
    let pt = decrypt(ENC_PLATFORM_TOOLS, &key);
    let un = decrypt(ENC_UNISOC, &key);
    extract_zip(&pt, &root.join("platform-tools"))?;
    extract_zip(&un, &root.join("Unisoc"))?;
    Ok(root)
}
