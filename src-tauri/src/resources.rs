use std::path::PathBuf;

// platform-tools, scrcpy and Unisoc ship as Tauri bundle resources next to the
// exe (see tauri.conf.json -> bundle.resources). No runtime extraction needed.
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