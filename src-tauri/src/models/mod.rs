use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectResult {
    pub mode: String,
    pub serial: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnisocPackage {
    pub id: String,
    pub name: String,
    pub exec_addr: u64,
    pub fdl1: String,
    pub fdl1_addr: u64,
    pub fdl2: String,
    pub fdl2_addr: u64,
    pub cboot: String,
    pub spl_loader_bk: Option<String>,
    pub misc_done: String,
    pub chsize_uboot: bool,
    pub tools_gen: String,
    pub erase_persist: bool,
    pub backup_partitions: Vec<String>,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub text: String,
    pub log_type: String,
    pub timestamp: u64,
}
