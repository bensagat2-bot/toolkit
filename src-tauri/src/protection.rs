use std::sync::Once;

static INIT: Once = Once::new();

#[cfg(target_os = "windows")]
mod win {
    extern "system" {
        fn IsDebuggerPresent() -> bool;
        fn CheckRemoteDebuggerPresent(h_process: *mut std::ffi::c_void, isDebugger: *mut bool) -> i32;
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn TerminateProcess(h_process: *mut std::ffi::c_void, u_exit_code: u32) -> i32;
    }

    pub fn anti_debug() -> bool {
        unsafe {
            if IsDebuggerPresent() {
                return true;
            }
            let mut present: bool = false;
            let _ = CheckRemoteDebuggerPresent(GetCurrentProcess(), &mut present);
            present
        }
    }

    pub fn kill() {
        unsafe {
            let _ = TerminateProcess(GetCurrentProcess(), 0xDEAD);
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod win {
    pub fn anti_debug() -> bool { false }
    pub fn kill() {}
}

pub fn init() {
    INIT.call_once(|| {
        if win::anti_debug() {
            win::kill();
        }
    });
}
