#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum CreaseRule {
    Unknown = crate::OpenSubdiv_v3_4_4_Sdc_Crease_Rule_RULE_UNKNOWN,
    Smooth = crate::OpenSubdiv_v3_4_4_Sdc_Crease_Rule_RULE_SMOOTH,
    Dart = crate::OpenSubdiv_v3_4_4_Sdc_Crease_Rule_RULE_DART,
    Create = crate::OpenSubdiv_v3_4_4_Sdc_Crease_Rule_RULE_CREASE,
    Corner = crate::OpenSubdiv_v3_4_4_Sdc_Crease_Rule_RULE_CORNER,
}
