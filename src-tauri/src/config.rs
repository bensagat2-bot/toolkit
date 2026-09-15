// Obfuscated strings - each byte is XOR'd with a key at compile time
macro_rules! obf {
    ($s:expr) => {{
        const KEY: u8 = 0x5A;
        const LEN: usize = $s.len();
        static OBF: [u8; LEN] = {
            let mut buf = [0u8; LEN];
            let src = $s.as_bytes();
            let mut i = 0;
            while i < LEN {
                buf[i] = src[i] ^ KEY;
                i += 1;
            }
            buf
        };
        let mut dec = [0u8; LEN];
        let mut i = 0;
        while i < LEN {
            dec[i] = OBF[i] ^ KEY;
            i += 1;
        }
        unsafe { std::str::from_utf8_unchecked(&dec) }
    }};
}

pub fn pds_api_base() -> &'static str {
    obf!("https://fra315.api.aliyunpds.com")
}

pub fn ota_mirror_host() -> &'static str {
    obf!("bkt-sgp-miui-ota-update-alisgp.oss-ap-southeast-1.aliyuncs.com")
}