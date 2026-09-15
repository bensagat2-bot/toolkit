// Central configuration for external service URLs and constants.
// Update these here instead of scattering them across modules.

use obfstr::obf;

pub fn pds_api_base() -> &'static str {
    obf!("https://fra315.api.aliyunpds.com")
}

pub fn ota_mirror_host() -> &'static str {
    obf!("bkt-sgp-miui-ota-update-alisgp.oss-ap-southeast-1.aliyuncs.com")
}
