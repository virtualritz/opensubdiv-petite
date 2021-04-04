use opensubdiv_sys as sys;

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum CreaseRule {
    Unknown = sys::OpenSubdiv_v3_4_4_Sdc_Crease_Rule_RULE_UNKNOWN,
    Smooth = sys::OpenSubdiv_v3_4_4_Sdc_Crease_Rule_RULE_SMOOTH,
    Dart = sys::OpenSubdiv_v3_4_4_Sdc_Crease_Rule_RULE_DART,
    Create = sys::OpenSubdiv_v3_4_4_Sdc_Crease_Rule_RULE_CREASE,
    Corner = sys::OpenSubdiv_v3_4_4_Sdc_Crease_Rule_RULE_CORNER,
}
