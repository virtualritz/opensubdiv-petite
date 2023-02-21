pub type PrimvarRefinerPtr = *mut crate::OpenSubdiv_v3_5_0_Far_PrimvarRefiner;

#[link(name = "osl-capi", kind = "static")]
extern "C" {
    pub fn PrimvarRefiner_create(
        tr: crate::TopologyRefinerPtr,
    ) -> PrimvarRefinerPtr;
    pub fn PrimvarRefiner_destroy(pr: PrimvarRefinerPtr);
    pub fn PrimvarRefiner_GetTopologyRefiner(pr: PrimvarRefinerPtr);
    pub fn PrimvarRefiner_Interpolate(
        pr: PrimvarRefinerPtr,
        num_elements: i32,
        level: i32,
        src: *const f32,
        dst: *mut f32,
    );
    pub fn PrimvarRefiner_InterpolateVarying(
        pr: PrimvarRefinerPtr,
        num_elements: i32,
        level: i32,
        src: *const f32,
        dst: *mut f32,
    );
    pub fn PrimvarRefiner_InterpolateFaceUniform(
        pr: PrimvarRefinerPtr,
        num_elements: i32,
        level: i32,
        src: *const f32,
        dst: *mut f32,
    );
    pub fn PrimvarRefiner_InterpolateFaceVarying(
        pr: PrimvarRefinerPtr,
        num_elements: i32,
        level: i32,
        src: *const f32,
        dst: *mut f32,
    );
}
