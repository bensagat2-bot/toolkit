use std::sync::Once;

static INIT: Once = Once::new();

macro_rules! obl {
    ($s:expr) => {{
        const K: u8 = 0x7C;
        const N: usize = $s.len();
        static O: [u8; N] = {
            let mut b = [0u8; N];
            let s = $s.as_bytes();
            let mut i = 0;
            while i < N {
                b[i] = s[i] ^ K;
                i += 1;
            }
            b
        };
        static D: std::sync::OnceLock<Vec<u8>> = std::sync::OnceLock::new();
        D.get_or_init(|| {
            let mut v = Vec::with_capacity(N + 1);
            for &b in &O {
                v.push(b ^ K);
            }
            v.push(0);
            v
        }).as_ptr()
    }};
}

#[cfg(target_os = "windows")]
#[allow(dead_code)]
mod win {
    extern "system" {
        fn IsDebuggerPresent() -> bool;
        fn CheckRemoteDebuggerPresent(h: *mut std::ffi::c_void, r: *mut bool) -> i32;
        fn GetCurrentProcess() -> *mut std::ffi::c_void;
        fn GetCurrentThread() -> *mut std::ffi::c_void;
        fn TerminateProcess(h: *mut std::ffi::c_void, c: u32) -> i32;
        fn GetModuleHandleA(m: *const u8) -> *mut std::ffi::c_void;
        fn GetProcAddress(m: *mut std::ffi::c_void, n: *const u8) -> *const ();
        fn GetThreadContext(h: *mut std::ffi::c_void, c: *mut std::ffi::c_void) -> i32;
        fn VirtualProtect(a: *mut std::ffi::c_void, s: usize, t: u32, o: *mut u32) -> i32;
    }

    type NtQIP = unsafe extern "system" fn(*mut std::ffi::c_void, u32, *mut std::ffi::c_void, u32, *mut u32) -> i32;
    type NtSIT = unsafe extern "system" fn(*mut std::ffi::c_void, u32, *mut std::ffi::c_void, u32) -> i32;
    type NtDE  = unsafe extern "system" fn(bool, *mut i64) -> i32;

    fn obf(sym: *const u8) -> Option<*const ()> {
        unsafe {
            let m = GetModuleHandleA(obl!("ntdll.dll"));
            if m.is_null() { return None; }
            let p = GetProcAddress(m, sym);
            if p.is_null() { None } else { Some(p) }
        }
    }

    static NQIP: std::sync::OnceLock<Option<NtQIP>> = std::sync::OnceLock::new();
    static NSIT: std::sync::OnceLock<Option<NtSIT>> = std::sync::OnceLock::new();
    static NDE:  std::sync::OnceLock<Option<NtDE>>  = std::sync::OnceLock::new();

    fn nqip() -> Option<NtQIP> {
        NQIP.get_or_init(|| {
            let p = obf(obl!("NtQueryInformationProcess"))?;
            Some(unsafe { std::mem::transmute::<*const (), NtQIP>(p) })
        }).copied()
    }
    fn nsit() -> Option<NtSIT> {
        NSIT.get_or_init(|| {
            let p = obf(obl!("NtSetInformationThread"))?;
            Some(unsafe { std::mem::transmute::<*const (), NtSIT>(p) })
        }).copied()
    }
    fn nde() -> Option<NtDE> {
        NDE.get_or_init(|| {
            let p = obf(obl!("NtDelayExecution"))?;
            Some(unsafe { std::mem::transmute::<*const (), NtDE>(p) })
        }).copied()
    }

    fn check_debug_port() -> bool {
        let f = nqip()?;
        let mut p: i32 = 0;
        let mut r: u32 = 0;
        unsafe { f(GetCurrentProcess(), 7, &mut p as *mut _ as *mut _, 4, &mut r) == 0 && p != 0 }
    }

    fn check_debug_object() -> bool {
        let f = nqip()?;
        let mut h: *mut std::ffi::c_void = std::ptr::null_mut();
        let mut r: u32 = 0;
        unsafe {
            f(GetCurrentProcess(), 30, &mut h as *mut _ as *mut _,
              std::mem::size_of::<*mut std::ffi::c_void>() as u32, &mut r) == 0 && !h.is_null()
        }
    }

    fn check_debug_flags() -> bool {
        let f = nqip()?;
        let mut g: i32 = 1;
        let mut r: u32 = 0;
        unsafe { f(GetCurrentProcess(), 0x1F, &mut g as *mut _ as *mut _, 4, &mut r) == 0 && g == 0 }
    }

    fn check_peb_flag() -> bool {
        let f = nqip()?;
        let mut i = [0u8; 48];
        let mut r: u32 = 0;
        let s = unsafe { f(GetCurrentProcess(), 0, i.as_mut_ptr() as *mut _, 48, &mut r) };
        if s != 0 { return false; }
        let peb = unsafe { *(i.as_ptr().add(8) as *const *mut std::ffi::c_void) };
        if peb.is_null() { return false; }
        let fl = unsafe { *(peb.add(0x2) as *const u8) };
        fl & 1 != 0
    }

    fn check_hw_breakpoints() -> bool {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let mut ctx: [u8; 0x300] = std::mem::zeroed();
            *(ctx.as_mut_ptr() as *mut u32) = 0x300;
            if GetThreadContext(GetCurrentThread(), ctx.as_mut_ptr() as *mut _) == 0 { return false; }
            let d0: usize = *(ctx.as_ptr().add(0x80) as *const usize);
            let d1: usize = *(ctx.as_ptr().add(0x88) as *const usize);
            let d2: usize = *(ctx.as_ptr().add(0x90) as *const usize);
            let d3: usize = *(ctx.as_ptr().add(0x98) as *const usize);
            d0 | d1 | d2 | d3 != 0
        }
        #[cfg(not(target_arch = "x86_64"))] { false }
    }

    fn hide_from_debugger() {
        if let Some(f) = nsit() {
            unsafe { let _ = f(GetCurrentThread(), 0x11, std::ptr::null_mut(), 0); }
        }
    }

    fn check_timing() -> bool {
        use std::time::{Duration, Instant};
        let f = nde()?;
        let iv: i64 = -10000;
        let st = Instant::now();
        unsafe { let _ = f(false, &iv as *const _ as *mut i64); }
        st.elapsed() > Duration::from_millis(10)
    }

    fn rdtsc_timing() -> bool {
        #[cfg(target_arch = "x86_64")]
        unsafe {
            let mut d = 0u64;
            std::arch::asm!("rdtsc", out("eax") _, out("edx") _);
            let (l1, h1): (u32, u32);
            std::arch::asm!("rdtsc", out("eax") l1, out("edx") h1);
            let s = ((h1 as u64) << 32) | l1 as u64;
            for _ in 0..500 { d ^= !d; }
            std::hint::black_box(d);
            let (l2, h2): (u32, u32);
            std::arch::asm!("rdtsc", out("eax") l2, out("edx") h2);
            let e = ((h2 as u64) << 32) | l2 as u64;
            e.wrapping_sub(s) > 30000
        }
        #[cfg(not(target_arch = "x86_64"))] { false }
    }

    fn crc32(base: *const u8, len: usize) -> u32 {
        let mut c = 0xFFFFFFFFu32;
        let n = len.min(4 * 1024 * 1024);
        for i in 0..n {
            let b = unsafe { *(base.add(i)) as u32 };
            c ^= b << 24;
            for _ in 0..8 { c = if c & 0x80000000 != 0 { (c << 1) ^ 0x04C11DB7 } else { c << 1 }; }
        }
        c ^ 0xFFFFFFFF
    }

    fn checksum_text() -> u32 {
        unsafe {
            let b = GetModuleHandleA(std::ptr::null());
            if b.is_null() { return 0; }
            let nt = *(b.add(0x3C) as *const u32);
            if nt == 0 { return 0; }
            let h = b.add(nt as usize);
            let sh = h.add(0x18);
            let sz = *(sh.add(0x10) as *const u32) as usize;
            let sa = b.add(*(sh.add(0x14) as *const u32) as usize);
            crc32(sa as *const u8, sz)
        }
    }

    fn xor_obfuscate_section() {
        unsafe {
            let b = GetModuleHandleA(std::ptr::null());
            if b.is_null() { return; }
            let nt = *(b.add(0x3C) as *const u32);
            if nt == 0 { return; }
            let h = b.add(nt as usize);
            let nsh = *(h.add(6) as *const u16) as usize;
            let opt = h.add(0x18);
            let sh = opt.add(*(h.add(0x10) as *const u16) as usize);
            for i in 1..nsh {
                let s = sh.add(i * 40);
                let c = *(s.add(8) as *const u32);
                let a = b.add(*(s.add(20) as *const u32) as usize);
                let mut o = 0u32;
                if VirtualProtect(a as *mut _, c as usize, 0x04, &mut o) != 0 {
                    for j in 0..c.min(0x200) {
                        let p = a.add(j as usize) as *mut u8;
                        *p ^= (j as u8).wrapping_mul(0x1B);
                    }
                    let _ = VirtualProtect(a as *mut _, c as usize, o, &mut o);
                }
            }
        }
    }

    fn anti_dump_peb() {
        let f = match nqip() { Some(f) => f, None => return };
        let mut i = [0u8; 48];
        let mut r: u32 = 0;
        unsafe {
            if f(GetCurrentProcess(), 0, i.as_mut_ptr() as *mut _, 48, &mut r) != 0 { return; }
            let peb = *(i.as_ptr().add(8) as *const *mut std::ffi::c_void);
            if peb.is_null() { return; }
            let bf = peb.add(0x2) as *mut u8;
            *bf &= !1u8;
        }
    }

    pub fn run_checks() -> Vec<&'static str> {
        let mut w = Vec::new();
        let mut push = |s: &'static str, ok: bool| { if ok { w.push(s); } };

        push("IsDebuggerPresent", unsafe { IsDebuggerPresent() });
        let mut p = false;
        unsafe { let _ = CheckRemoteDebuggerPresent(GetCurrentProcess(), &mut p); }
        push("RemoteDebugger", p);
        push("DebugPort", check_debug_port());
        push("DebugObject", check_debug_object());
        push("DebugFlags", check_debug_flags());
        push("PEBFlag", check_peb_flag());
        push("HWBreakpoints", check_hw_breakpoints());
        push("Timing", check_timing());
        push("RDTSC", rdtsc_timing());
        push("Integrity", !verify_integrity());

        hide_from_debugger();
        anti_dump_peb();

        w
    }

    pub fn verify_integrity() -> bool {
        static E: std::sync::OnceLock<u32> = std::sync::OnceLock::new();
        let e = E.get_or_init(|| checksum_text());
        *e == checksum_text()
    }

    pub fn scrub() {
        xor_obfuscate_section();
        anti_dump_peb();
        hide_from_debugger();
    }

    pub fn opaque_predicate() -> u64 {
        let a = 0x6FDBu64;
        let b = a.wrapping_mul(0x9E37);
        let c = b ^ 0xDEAD;
        let d = c.wrapping_add(0xBEEF);
        let e = d.rotate_left(7);
        let f = e.wrapping_sub(0x1FDB);
        f ^ 0xCAFE
    }

    pub fn junk() {
        let mut x: u64 = opaque_predicate();
        for i in 0..7 {
            x = x.wrapping_mul(0x9E3779B9);
            x ^= x.rotate_right(13);
            x = x.wrapping_add((i as u64).wrapping_mul(0x5A5A));
        }
        if x == 0x4242 { std::hint::black_box(x); }
        let y = x.wrapping_mul(0x3C3C3C3C);
        let z = y.rotate_left(17) ^ 0xA5A5A5A5;
        std::hint::black_box(z);
    }
}

#[cfg(not(target_os = "windows"))]
mod win {
    pub fn run_checks() -> Vec<&'static str> { Vec::new() }
    pub fn verify_integrity() -> bool { true }
    pub fn scrub() {}
    pub fn opaque_predicate() -> u64 { 0xCAFE }
    pub fn junk() {}
}

pub fn init() {
    INIT.call_once(|| {
        win::junk();
        let warnings = win::run_checks();
        for w in &warnings {
            crate::log::write(&format!("[PROTECT] detected: {}", w));
        }
        win::scrub();
        if !warnings.is_empty() {
            crate::log::write("[PROTECT] some checks triggered but continuing");
        }
    });
}

pub fn integrity_ok() -> bool {
    win::verify_integrity()
}