#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Scheme {
    Bilinear,
    CatmullClark,
    Loop,
}

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum Split {
    ToQuads,
    ToTris,
    Hybrid,
}
