use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

static LOGGER: Mutex<Option<Logger>> = Mutex::new(None);

struct Logger {
    file: fs::File,
}

pub fn init() {
    let path = log_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    let file = match fs::File::create(&path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let mut log = Logger { file };
    let _ = writeln!(log.file, "[V1Per] Log started at {:?}", std::time::SystemTime::now());
    let _ = LOGGER.lock().map(|mut g| *g = Some(log));
}

fn log_path() -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("V1Per").join("logs").join("toolkit.log")
}

pub fn write(msg: &str) {
    if let Ok(mut guard) = LOGGER.lock() {
        if let Some(ref mut log) = *guard {
            let _ = writeln!(log.file, "[{}] {}", chrono_or_elapsed(), msg);
        }
    }
}

fn chrono_or_elapsed() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format!("{:02}:{:02}:{:02}", now.as_secs() / 3600 % 24, now.as_secs() / 60 % 60, now.as_secs() % 60)
}