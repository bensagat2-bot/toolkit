use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

const PLATFORM_TOOLS_ZIP: &[u8] = include_bytes!("../embed/platform-tools.zip");
const UNISOC_ZIP: &[u8] = include_bytes!("../embed/unisoc.zip");
const SCRCPY_ZIP: &[u8] = include_bytes!("../embed/scrcpy.zip");

fn cache_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| std::env::temp_dir())
        .join("V1PerServicing")
        .join("cache")
}

fn extract_zip(data: &[u8], dest: &Path) -> Result<(), String> {
    // Use a marker file so a partial/stale dir triggers a clean re-extract.
    let done_marker = dest.join(".complete");
    if done_marker.exists() {
        return Ok(());
    }
    if dest.exists() {
        fs::remove_dir_all(dest).map_err(|e| format!("Failed to clear stale cache: {e}"))?;
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
    fs::write(&done_marker, "ok").map_err(|e| format!("Failed to write marker: {e}"))?;
    Ok(())
}

pub fn extract_all() -> Result<PathBuf, String> {
    let root = cache_dir();
    fs::create_dir_all(&root).map_err(|e| format!("Failed to create cache dir: {e}"))?;
    extract_zip(PLATFORM_TOOLS_ZIP, &root.join("platform-tools"))?;
    extract_zip(UNISOC_ZIP, &root.join("Unisoc"))?;
    extract_zip(SCRCPY_ZIP, &root.join("scrcpy"))?;
    Ok(root)
}
