use super::topology_refiner::TopologyRefinerPtr;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct PrimvarRefiner_obj {
    _unused: [u8; 0],
}
pub type PrimvarRefinerPtr = *mut crate::OpenSubdiv_v3_4_4_Far_PrimvarRefiner;

extern "C" {
    pub fn PrimvarRefiner_create(
        tr: *mut crate::OpenSubdiv_v3_4_4_Far_TopologyRefiner,
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
