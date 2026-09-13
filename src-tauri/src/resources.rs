use std::path::{Path, PathBuf};
use std::sync::OnceLock;

// Matches src-tauri/encrypt_resources.py: V 1 P e r S e r v I c e
const KEY: [u8; 12] = [0x56, 0x31, 0x50, 0x45, 0x72, 0x53, 0x65, 0x72, 0x76, 0x49, 0x63, 0x65];

static PLATFORM_TOOLS_ENC: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/platform-tools.enc"));
static SCRCPY_ENC: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/scrcpy.enc"));
static UNISOC_ENC: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/unisoc.enc"));

static RESOURCE_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn set_resource_dir(path: PathBuf) {
    let _ = RESOURCE_DIR.set(path);
}

fn decrypt(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ KEY[i % KEY.len()])
        .collect()
}

// Versioned per-user cache so each release extracts its own tools and prunes
// the previous version.
fn cache_root() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("V1Per").join("tools").join(env!("CARGO_PKG_VERSION"))
}

fn extract(cache_name: &str, enc: &[u8]) -> Option<PathBuf> {
    let dest = cache_root().join(cache_name);
    let marker = dest.join(".extracted");
    if marker.exists() {
        return Some(dest);
    }
    let data = decrypt(enc);
    let cursor = std::io::Cursor::new(data);
    let mut archive = zip::ZipArchive::new(cursor).ok()?;
    std::fs::create_dir_all(&dest).ok()?;
    for i in 0..archive.len() {
        let mut file = archive.by_index(i).ok()?;
        let out_path = match file.enclosed_name() {
            Some(p) => dest.join(p),
            None => continue,
        };
        if file.is_dir() {
            std::fs::create_dir_all(&out_path).ok();
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent).ok();
            }
            let mut f = std::fs::File::create(&out_path).ok()?;
            std::io::copy(&mut file, &mut f).ok();
        }
    }
    let _ = std::fs::write(&marker, b"ok");
    Some(dest)
}

// Lazy extraction: returns the extracted folder for a bundled tool, extracting
// it from the embedded encrypted blob on first use only.
pub fn ensure_tool_dir(folder: &str) -> Option<PathBuf> {
    match folder {
        "platform-tools" => extract("platform-tools", PLATFORM_TOOLS_ENC),
        "scrcpy" => extract("scrcpy", SCRCPY_ENC),
        "Unisoc" => extract("Unisoc", UNISOC_ENC),
        _ => None,
    }
}

// platform-tools, scrcpy and Unisoc are embedded into the exe now (see
// build.rs). bundle_dir() is kept as a dev/bundle fallback location.
pub fn bundle_dir() -> PathBuf {
    // Walk up from the exe looking for the `resources` folder (installed layout).
    let mut dir = std::env::current_exe()
        .ok()
        .and_then(|e| e.parent().map(|p| p.to_path_buf()));
    for _ in 0..8 {
        if let Some(d) = dir {
            let r = d.join("resources");
            if r.join("Unisoc").exists() {
                return r;
            }
            dir = d.parent().map(|p| p.to_path_buf());
        }
    }
    // Dev fallback: repo layout.
    std::path::Path::new("src-tauri").join("resources")
}

// Prunes older versioned tool caches, keeping only the current version.
pub fn prune_old_caches() {
    let Some(appdata) = std::env::var_os("LOCALAPPDATA") else {
        return;
    };
    let root = PathBuf::from(appdata).join("V1Per").join("tools");
    let Ok(entries) = std::fs::read_dir(&root) else {
        return;
    };
    let current = env!("CARGO_PKG_VERSION");
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name != current && entry.path().is_dir() {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}