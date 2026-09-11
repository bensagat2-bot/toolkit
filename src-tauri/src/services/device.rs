use serde::{Deserialize, Serialize};
use super::adb;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectResult {
    pub mode: String,
    pub serial: Option<String>,
}

pub async fn check_adb() -> bool {
    let output = adb::run_adb(&["version"]).await;
    !output.is_empty() && !output.contains("not found")
}

pub async fn check_fastboot() -> bool {
    let output = adb::run_fastboot(&["--version"]).await;
    !output.is_empty() && !output.contains("not found")
}

pub async fn get_info() -> String {
    adb::run_adb(&["shell", "getprop"]).await
}

pub async fn detect() -> DetectResult {
    let (mode, serial) = adb::detect().await;
    DetectResult { mode, serial }
}
