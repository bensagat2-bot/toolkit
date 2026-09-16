use std::sync::Once;

static INIT: Once = Once::new();

#[cfg(target_os = "windows")]
#[allow(dead_code)]
mod win {
    pub fn run_checks() -> Vec<&'static str> {
        Vec::new()
    }

    pub fn verify_integrity() -> bool {
        true
    }

    pub fn scrub() {}

    pub fn junk() {}
}

#[cfg(not(target_os = "windows"))]
mod win {
    pub fn run_checks() -> Vec<&'static str> { Vec::new() }
    pub fn verify_integrity() -> bool { true }
    pub fn scrub() {}
    pub fn junk() {}
}

pub fn init() {
    INIT.call_once(|| {
        win::junk();
        let warnings = win::run_checks();
        if !warnings.is_empty() {
            crate::log::write("[PROTECT] checks triggered (non-fatal)");
        }
        win::scrub();
    });
}

pub fn integrity_ok() -> bool {
    true
}