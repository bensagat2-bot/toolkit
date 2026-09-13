use std::path::{Path, PathBuf};

// Matches src-tauri/encrypt_resources.py: V 1 P e r S e r v I c e
const KEY: [u8; 12] = [0x56, 0x31, 0x50, 0x45, 0x72, 0x53, 0x65, 0x72, 0x76, 0x49, 0x63, 0x65];

fn xor(data: &[u8]) -> Vec<u8> {
    data.iter()
        .enumerate()
        .map(|(i, b)| b ^ KEY[i % KEY.len()])
        .collect()
}

fn add_tree(
    zip: &mut zip::ZipWriter<std::fs::File>,
    base: &Path,
    dir: &Path,
    opts: &zip::write::SimpleFileOptions,
) -> std::io::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let rel = path
            .strip_prefix(base)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            zip.add_directory(format!("{rel}/"), opts.clone())?;
            add_tree(zip, base, &path, opts)?;
        } else {
            zip.start_file(rel, opts.clone())?;
            let mut f = std::fs::File::open(&path)?;
            std::io::copy(&mut f, zip)?;
        }
    }
    Ok(())
}

fn zip_dir(dir: &Path, out: &Path) -> std::io::Result<()> {
    let file = std::fs::File::create(out)?;
    let mut zip = zip::ZipWriter::new(file);
    let opts = zip::write::SimpleFileOptions::default();
    add_tree(&mut zip, dir, dir, &opts)?;
    zip.finish()?;
    Ok(())
}

fn main() {
    // Require administrator elevation. Windows shows the UAC yes/no prompt and
    // overlays a shield badge on the exe icon automatically.
    let attrs = tauri_build::Attributes::new()
        .windows_attributes(tauri_build::WindowsAttributes::new().app_manifest("app.manifest"));
    tauri_build::build_attributes(attrs);

    // Bundle the three tool folders into encrypted blobs embedded in the
    // binary. They are extracted on demand to %LOCALAPPDATA%\V1Per\tools so
    // the app can ship as a single clean portable exe.
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    for (rel, name) in [
        ("platform-tools", "platform-tools"),
        ("scrcpy", "scrcpy"),
        ("Unisoc", "unisoc"),
    ] {
        let src = Path::new("resources").join(rel);
        if !src.exists() {
            continue;
        }
        let zip_path = out_dir.join(format!("{name}.zip"));
        zip_dir(&src, &zip_path).expect("zip resources");
        let data = std::fs::read(&zip_path).expect("read zip");
        std::fs::write(out_dir.join(format!("{name}.enc")), xor(&data)).expect("encrypt resources");
        println!("cargo:rerun-if-changed=resources/{rel}");
    }
}