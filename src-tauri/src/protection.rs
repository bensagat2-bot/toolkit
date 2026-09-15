use std::sync::Once;

static INIT: Once = Once::new();

#[cfg(target_os = "windows")]
mod win {
    use std::time::{Duration, Instant};

    extern "system" {
        fn IsDebuggerPresent() -> bool;
        fn CheckRemoteDebuggerPresent(h_process: *mut std::ffi::c_void, is_debugger: *mut bool) -> i32;
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn GetCurrentThread() -> *mut std::ffi::c_void;
        fn TerminateProcess(h_process: *mut std::ffi::c_void, u_exit_code: u32) -> i32;
        fn NtRaiseHardError(
            error_status: u32, number_of_parameters: u32,
            unicode_string_parameter_mask: u32,
            parameters: *mut std::ffi::c_void,
            response: *mut u32,
        ) -> i32;
        fn RtlAdjustPrivilege(
            privilege: u32, enable: bool, current_thread: bool,
            old_value: *mut bool,
        ) -> i32;
        fn GetModuleHandleA(lp_module_name: *const u8) -> *mut std::ffi::c_void;
        fn GetProcAddress(h_module: *mut std::ffi::c_void, lp_proc_name: *const u8) -> *const ();
    }

    type NtQueryInfoProc = unsafe extern "system" fn(
        *mut std::ffi::c_void, u32, *mut std::ffi::c_void, u32, *mut u32,
    ) -> i32;
    type NtSetInfoThread = unsafe extern "system" fn(
        *mut std::ffi::c_void, u32, *mut std::ffi::c_void, u32,
    ) -> i32;
    type NtDelayExec = unsafe extern "system" fn(bool, *mut i64) -> i32;

    fn resolve_ntdll(func: &str) -> Option<*const ()> {
        unsafe {
            let ntdll = [0x6Eu8, 0x74, 0x64, 0x6C, 0x6C, 0x2E, 0x64, 0x6C, 0x6C, 0x00];
            let mod_h = GetModuleHandleA(ntdll.as_ptr());
            if mod_h.is_null() { return None; }
            let cname = std::ffi::CString::new(func).ok()?;
            let ptr = GetProcAddress(mod_h, cname.as_ptr());
            if ptr.is_null() { None } else { Some(ptr) }
        }
    }

    fn check_debug_port() -> bool {
        let Some(ptr) = resolve_ntdll("NtQueryInformationProcess") else { return false };
        let func: NtQueryInfoProc = unsafe { std::mem::transmute(ptr) };
        let mut port: i32 = 0;
        let mut ret_len: u32 = 0;
        let status = unsafe {
            func(GetCurrentProcess(), 7, &mut port as *mut _ as *mut _, 4, &mut ret_len)
        };
        status == 0 && port != 0
    }

    fn check_debug_object() -> bool {
        let Some(ptr) = resolve_ntdll("NtQueryInformationProcess") else { return false };
        let func: NtQueryInfoProc = unsafe { std::mem::transmute(ptr) };
        let mut handle: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut ret_len: u32 = 0;
        let status = unsafe {
            func(GetCurrentProcess(), 30, &mut handle as *mut _ as *mut _,
                std::mem::size_of::<*mut std::ffi::c_void>() as u32, &mut ret_len)
        };
        status == 0 && !handle.is_null()
    }

    fn hide_from_debugger() {
        let Some(ptr) = resolve_ntdll("NtSetInformationThread") else { return };
        let func: NtSetInfoThread = unsafe { std::mem::transmute(ptr) };
        unsafe { let _ = func(GetCurrentThread(), 0x11, std::ptr::null_mut(), 0); }
    }

    fn check_timing() -> bool {
        let Some(ptr) = resolve_ntdll("NtDelayExecution") else { return false };
        let func: NtDelayExec = unsafe { std::mem::transmute(ptr) };
        let interval: i64 = -10000;
        let start = Instant::now();
        unsafe { let _ = func(false, &interval as *const _ as *mut i64); }
        start.elapsed() > Duration::from_millis(10)
    }

    fn rdtsc_timing() -> bool {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let start: u64;
            std::arch::asm!("rdtsc", "mov {}, eax", out(reg) start, options(nostack, att_syntax));
            let mut dummy = 0u64;
            for _ in 0..1000 { dummy ^= i64::MAX as u64; }
            let end: u64;
            std::arch::asm!("rdtsc", "mov {}, eax", out(reg) end, options(nostack, att_syntax));
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            let _ = dummy;
            (end - start) > 50000
        }
        #[cfg(not(target_arch = "x86_64"))]
        { false }
    }

    fn checksum_text() -> u32 {
        unsafe {
            let base = GetModuleHandleA(std::ptr::null());
            if base.is_null() { return 0; }
            let nt_offset = *(base.add(0x3C) as *const u32);
            if nt_offset == 0 { return 0; }
            let nt = base.add(nt_offset as usize);
            let sect_hdr = nt.add(0x18);
            let sect_size = *(sect_hdr.add(0x10) as *const u32) as usize;
            let sect_addr = base.add(*(sect_hdr.add(0x14) as *const u32) as usize);
            let mut crc = 0xFFFFFFFFu32;
            let len = sect_size.min(4 * 1024 * 1024);
            for i in 0..len {
                let byte = *(sect_addr.add(i) as *const u8) as u32;
                crc ^= byte << 24;
                for _ in 0..8 {
                    if crc & 0x80000000 != 0 { crc = (crc << 1) ^ 0x04C11DB7; }
                    else { crc <<= 1; }
                }
            }
            crc ^ 0xFFFFFFFF
        }
    }

    static TEXT_CRC: std::sync::OnceLock<u32> = std::sync::OnceLock::new();

    pub fn verify_integrity() -> bool {
        let expected = TEXT_CRC.get_or_init(|| checksum_text());
        *expected == checksum_text()
    }

    pub fn opaque_true() -> bool {
        let a = (Instant::now().elapsed().as_nanos() & 3) as u64;
        let b = a ^ 0xFFFF;
        let c = a.wrapping_add(b);
        c.wrapping_sub(0xFFFF) == 0
    }

    fn check_vm() -> bool {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let cpuid = std::arch::x86_64::__cpuid(1);
            (cpuid.ecx >> 31) & 1 != 0
        }
        #[cfg(not(target_arch = "x86_64"))]
        { false }
    }

    pub fn run_checks() -> bool {
        let mut detected = false;

        if unsafe { IsDebuggerPresent() } { detected = true; }
        let mut present: bool = false;
        unsafe { let _ = CheckRemoteDebuggerPresent(GetCurrentProcess(), &mut present); }
        if present { detected = true; }

        if check_debug_port() { detected = true; }
        if check_debug_object() { detected = true; }
        if check_timing() { detected = true; }
        if rdtsc_timing() { detected = true; }
        if !verify_integrity() { detected = true; }
        if check_vm() { detected = true; }

        hide_from_debugger();
        detected
    }

    pub fn kill() {
        unsafe {
            let mut response: u32 = 0;
            let _ = NtRaiseHardError(0xC0000354, 0, 0, std::ptr::null_mut(), &mut response);
            let _ = TerminateProcess(GetCurrentProcess(), 0xDEAD);
            let mut old = false;
            let _ = RtlAdjustPrivilege(0x13, false, false, &mut old);
        }
    }

    pub fn junk_code() {
        let now = Instant::now();
        let _elapsed = now.elapsed().as_nanos();
        let mut x: u64 = 0xDEADBEEF;
        for _ in 0..3 {
            x = x.wrapping_mul(0x9E3779B9);
            x ^= x >> 16;
        }
        if x == 0 { std::hint::black_box(x); }
    }
}

#[cfg(not(target_os = "windows"))]
mod win {
    pub fn run_checks() -> bool { false }
    pub fn kill() {}
    pub fn opaque_true() -> bool { true }
    pub fn verify_integrity() -> bool { true }
    pub fn junk_code() {}
}

pub fn init() {
    INIT.call_once(|| {
        win::junk_code();
        if win::run_checks() {
            win::kill();
        }
    });
}

pub fn integrity_ok() -> bool {
    win::verify_integrity()
}