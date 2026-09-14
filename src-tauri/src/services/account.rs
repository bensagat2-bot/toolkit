use std::process::Command;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

fn run_cmd(program: &str, args: &[&str]) -> String {
    let mut cmd = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    cmd.args(args);
    cmd.output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_default()
}

// Full PC fingerprint reported on register/login so the admin panel can show
// where an account is used. Field names match what the backend stores.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PcInfo {
    pub machine_guid: String,
    pub disk_serial: String,
    pub board_serial: String,
    pub mac: String,
    pub cpu: String,
    pub os_version: String,
    pub ram: String,
    pub device_model: String,
}

#[cfg(target_os = "windows")]
fn powershell_fingerprint_script() -> &'static str {
    r#"
$out = [ordered]@{}
try { $out.machine_guid = (Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Cryptography' -Name MachineGuid).MachineGuid } catch { $out.machine_guid = '' }
try { $out.disk_serial = (((Get-CimInstance Win32_DiskDrive | Select-Object -First 1).SerialNumber).Trim()) } catch { $out.disk_serial = '' }
try { $out.board_serial = (((Get-CimInstance Win32_BaseBoard | Select-Object -First 1).SerialNumber).Trim()) } catch { $out.board_serial = '' }
try { $out.mac = ((Get-NetAdapter -Physical | Where-Object { $_.Status -eq 'Up' } | Select-Object -First 1).MacAddress) } catch { $out.mac = '' }
try { $out.cpu = ((Get-CimInstance Win32_Processor | Select-Object -First 1).ProcessorId) } catch { $out.cpu = '' }
try { $os = Get-CimInstance Win32_OperatingSystem; $out.os_version = ($os.Caption + ' ' + $os.Version) } catch { $out.os_version = '' }
try { $cs = Get-CimInstance Win32_ComputerSystem; $out.ram = [math]::Round($cs.TotalPhysicalMemory / 1GB, 0).ToString(); $out.device_model = (($cs.Manufacturer + ' ' + $cs.Model).Trim()) } catch { $out.ram = ''; $out.device_model = '' }
$out | ConvertTo-Json -Compress
"#
}

#[cfg(target_os = "windows")]
fn read_pc_info() -> PcInfo {
    let script = powershell_fingerprint_script();
    let out = run_cmd(
        "powershell",
        &["-NoProfile", "-NonInteractive", "-Command", script],
    );
    if out.is_empty() {
        return PcInfo::default();
    }
    serde_json::from_str(&out).unwrap_or_default()
}

#[cfg(not(target_os = "windows"))]
fn read_pc_info() -> PcInfo {
    PcInfo::default()
}

static PC_INFO: OnceLock<PcInfo> = OnceLock::new();

pub fn get_pc_info() -> PcInfo {
    PC_INFO.get_or_init(read_pc_info).clone()
}

fn sha256_hex(data: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(data.as_bytes());
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{:02x}", b))
        .collect()
}

#[cfg(target_os = "windows")]
pub fn get_hwid() -> String {
    let info = get_pc_info();
    // Hardware-based (survives Windows reinstalls). MachineGuid is included so
    // a fresh format still yields a new id while a drive swap does not reset it.
    let seed = format!(
        "{}|{}|{}|{}",
        info.disk_serial, info.board_serial, info.mac, info.machine_guid
    );
    if seed.trim_matches('|').is_empty() {
        crate::log::write("HWID: no hardware data available, using UNKNOWN-HWID");
        return "UNKNOWN-HWID".to_string();
    }
    let hwid = sha256_hex(&seed);
    crate::log::write(&format!("HWID generated ({})", &hwid[..8]));
    hwid
}

#[cfg(not(target_os = "windows"))]
pub fn get_hwid() -> String {
    let out = run_cmd("hostname", &[]);
    if out.is_empty() {
        "UNKNOWN-HWID".to_string()
    } else {
        sha256_hex(&out)
    }
}

pub fn get_device_model() -> String {
    let info = get_pc_info();
    if !info.device_model.is_empty() {
        return info.device_model;
    }
    "Unknown device".to_string()
}