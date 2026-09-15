use std::sync::Once;

static INIT: Once = Once::new();

#[cfg(target_os = "windows")]
#[allow(dead_code)]
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
        fn GetThreadContext(h_thread: *mut std::ffi::c_void, context: *mut std::ffi::c_void) -> i32;
        fn OpenThread(dw_desired_access: u32, inherit_handle: bool, thread_id: u32) -> *mut std::ffi::c_void;
        fn CloseHandle(h_object: *mut std::ffi::c_void) -> i32;
        fn NtCloseHandle(h: *mut std::ffi::c_void) -> i32;
    }

    type NtQueryInfoProc = unsafe extern "system" fn(
        *mut std::ffi::c_void, u32, *mut std::ffi::c_void, u32, *mut u32,
    ) -> i32;
    type NtSetInfoThread = unsafe extern "system" fn(
        *mut std::ffi::c_void, u32, *mut std::ffi::c_void, u32,
    ) -> i32;
    type NtDelayExec = unsafe extern "system" fn(bool, *mut i64) -> i32;
    type NtQueryInfoThread = unsafe extern "system" fn(
        *mut std::ffi::c_void, u32, *mut std::ffi::c_void, u32, *mut u32,
    ) -> i32;

    fn resolve_ntdll(func: &str) -> Option<*const ()> {
        unsafe {
            let ntdll = [0x6Eu8, 0x74, 0x64, 0x6C, 0x6C, 0x2E, 0x64, 0x6C, 0x6C, 0x00];
            let mod_h = GetModuleHandleA(ntdll.as_ptr());
            if mod_h.is_null() { return None; }
            let cname = std::ffi::CString::new(func).ok()?;
            let ptr = GetProcAddress(mod_h, cname.as_ptr() as *const u8);
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

    fn check_debug_flags() -> bool {
        let Some(ptr) = resolve_ntdll("NtQueryInformationProcess") else { return false };
        let func: NtQueryInfoProc = unsafe { std::mem::transmute(ptr) };
        let mut flags: i32 = 1;
        let mut ret_len: u32 = 0;
        let status = unsafe {
            func(GetCurrentProcess(), 0x1F, &mut flags as *mut _ as *mut _,
                4, &mut ret_len)
        };
        status == 0 && flags == 0
    }

    fn check_peb_flag() -> bool {
        let Some(ptr) = resolve_ntdll("NtQueryInformationProcess") else { return false };
        let func: NtQueryInfoProc = unsafe { std::mem::transmute(ptr) };
        let mut info = [0u8; 48];
        let mut ret_len: u32 = 0;
        let status = unsafe {
            func(GetCurrentProcess(), 0, info.as_mut_ptr() as *mut _,
                48, &mut ret_len)
        };
        if status != 0 { return false; }
        let peb_ptr = unsafe { *(info.as_ptr().add(8) as *const *mut std::ffi::c_void) };
        if peb_ptr.is_null() { return false; }
        let flag = unsafe { *(peb_ptr.add(0x2) as *const u8) };
        flag & 1 != 0
    }

    fn check_hw_breakpoints() -> bool {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let thread = GetCurrentThread();
            let mut ctx: [u8; 0x300] = std::mem::zeroed();
            let ctx_ptr = ctx.as_mut_ptr();
            *(ctx_ptr as *mut u32) = 0x300;
            if GetThreadContext(thread, ctx_ptr as *mut _) == 0 {
                return false;
            }
            let dr0: usize = *(ctx_ptr.add(0x80) as *const usize);
            let dr1: usize = *(ctx_ptr.add(0x88) as *const usize);
            let dr2: usize = *(ctx_ptr.add(0x90) as *const usize);
            let dr3: usize = *(ctx_ptr.add(0x98) as *const usize);
            dr0 != 0 || dr1 != 0 || dr2 != 0 || dr3 != 0
        }
        #[cfg(not(target_arch = "x86_64"))]
        { false }
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
            std::arch::asm!("rdtsc", "movzx {}, eax", out(reg) start, options(nostack));
            let mut dummy = 0u64;
            for _ in 0..1000 { dummy ^= i64::MAX as u64; }
            let end: u64;
            std::arch::asm!("rdtsc", "movzx {}, eax", out(reg) end, options(nostack));
            std::sync::atomic::compiler_fence(std::sync::atomic::Ordering::SeqCst);
            let _ = dummy;
            (end - start) > 50000
        }
        #[cfg(not(target_arch = "x86_64"))]
        { false }
    }

    fn checksum_range(base: *const u8, len: usize) -> u32 {
        let mut crc = 0xFFFFFFFFu32;
        let limit = len.min(4 * 1024 * 1024);
        for i in 0..limit {
            let byte = unsafe { *(base.add(i)) as u32 };
            crc ^= byte << 24;
            for _ in 0..8 {
                if crc & 0x80000000 != 0 { crc = (crc << 1) ^ 0x04C11DB7; }
                else { crc <<= 1; }
            }
        }
        crc ^ 0xFFFFFFFF
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
            checksum_range(sect_addr as *const u8, sect_size)
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

    #[allow(unused_unsafe)]
    fn check_vm() -> bool {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let cpuid = std::arch::x86_64::__cpuid(1);
            (cpuid.ecx >> 31) & 1 != 0
        }
        #[cfg(not(target_arch = "x86_64"))]
        { false }
    }

    fn check_vm_hypervisor_brand() -> bool {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let res = std::arch::x86_64::__cpuid_count(0x40000001, 0);
            let mut brand = [0u8; 16];
            std::ptr::copy_nonoverlapping(&res.ebx as *const u32 as *const u8, brand.as_mut_ptr(), 4);
            std::ptr::copy_nonoverlapping(&res.ecx as *const u32 as *const u8, brand.as_mut_ptr().add(4), 4);
            std::ptr::copy_nonoverlapping(&res.edx as *const u32 as *const u8, brand.as_mut_ptr().add(8), 4);
            let s = String::from_utf8_lossy(&brand);
            s == "Microsoft Hv" || s == "VMwareVMware" || s == "KVMKVM  KVM"
        }
        #[cfg(not(target_arch = "x86_64"))]
        { false }
    }

    fn check_anti_hook(func_name: &str, expected_bytes: &[u8]) -> bool {
        let Some(ptr) = resolve_ntdll(func_name) else { return false };
        unsafe {
            let ptr = ptr as *const u8;
            let mut matches = true;
            for i in 0..expected_bytes.len() {
                if *ptr.add(i) != expected_bytes[i] {
                    matches = false;
                    break;
                }
            }
            !matches
        }
    }

    fn check_ntdll_hooks() -> bool {
        let hooked1 = check_anti_hook("NtQueryInformationProcess",
            &[0x4C, 0x8B, 0xD1, 0xB8, 0x17, 0x00, 0x00, 0x00, 0xF6, 0x04, 0x25]);
        let hooked2 = check_anti_hook("NtSetInformationThread",
            &[0x4C, 0x8B, 0xD1, 0xB8, 0x0D, 0x00, 0x00, 0x00, 0xF6, 0x04, 0x25]);
        let hooked3 = check_anti_hook("NtDelayExecution",
            &[0x4C, 0x8B, 0xD1, 0xB8, 0x34, 0x00, 0x00, 0x00, 0xF6, 0x04, 0x25]);
        hooked1 || hooked2 || hooked3
    }

    pub fn run_checks() -> bool {
        let mut detected = false;

        if unsafe { IsDebuggerPresent() } { detected = true; }
        let mut present: bool = false;
        unsafe { let _ = CheckRemoteDebuggerPresent(GetCurrentProcess(), &mut present); }
        if present { detected = true; }

        if check_debug_port() { detected = true; }
        if check_debug_object() { detected = true; }
        if check_debug_flags() { detected = true; }
        if check_peb_flag() { detected = true; }
        if check_hw_breakpoints() { detected = true; }
        if check_timing() { detected = true; }
        if rdtsc_timing() { detected = true; }
        if !verify_integrity() { detected = true; }
        if check_vm() { detected = true; }
        if check_vm_hypervisor_brand() { detected = true; }
        if check_ntdll_hooks() { detected = true; }

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
        let mut y = 0xCAFEBABEu64;
        y = y.wrapping_mul(0x9E3779B9);
        y ^= y >> 13;
        y = y.wrapping_mul(0x9E3779B9);
        y ^= y >> 17;
        std::hint::black_box(y);
        let mut z: u64 = 0;
        for i in 0..10 {
            z = z.wrapping_add((i as u64).wrapping_mul(0x9E3779B9));
            z = z.rotate_left(5);
        }
        std::hint::black_box(z);
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