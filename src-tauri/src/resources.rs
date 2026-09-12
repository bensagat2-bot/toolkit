use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

const PLATFORM_TOOLS_ZIP: &[u8] = include_bytes!("../embed/platform-tools.zip");
const UNISOC_ZIP: &[u8] = include_bytes!("../embed/unisoc.zip");

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
    let root = cache_dir();
    fs::create_dir_all(&root).map_err(|e| format!("Failed to create cache dir: {e}"))?;
    extract_zip(PLATFORM_TOOLS_ZIP, &root.join("platform-tools"))?;
    extract_zip(UNISOC_ZIP, &root.join("Unisoc"))?;
    Ok(root)
}
