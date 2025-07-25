use crate::vtr::types::*;

// FIXME: figure out why bindgen doesn't generate this struct
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StencilTableOptions {
    pub interpolation_mode: u32,
    pub generate_offsets: u32,
    pub generate_control_vertices: u32,
    pub generate_intermediate_levels: u32,
    pub factorize_intermediate_levels: u32,
    pub max_level: u32,
    pub face_varying_channel: u32,
}

pub type Stencil = crate::OpenSubdiv_v3_6_1_Far_StencilReal<f32>;
pub type StencilTable = crate::OpenSubdiv_v3_6_1_Far_StencilTableReal;
pub type StencilTablePtr = *mut StencilTable;

#[link(name = "osd-capi", kind = "static")]
extern "C" {
    pub fn StencilTableFactory_Create(
        refiner: *mut crate::OpenSubdiv_v3_6_1_Far_TopologyRefiner,
        options: StencilTableOptions,
    ) -> StencilTablePtr;

    pub fn StencilTable_destroy(st: StencilTablePtr);
    /// Returns the number of stencils in the table
    pub fn StencilTable_GetNumStencils(st: StencilTablePtr) -> u32;
    /// Returns the number of control vertices indexed in the table
    pub fn StencilTable_GetNumControlVertices(st: StencilTablePtr) -> u32;
    /// Returns a Stencil at index i in the table
    pub fn StencilTable_GetStencil(st: StencilTablePtr, index: Index) -> Stencil;
    /// Returns the number of control vertices of each stencil in the table
    pub fn StencilTable_GetSizes(st: StencilTablePtr) -> IntVectorRef;
    /// Returns the offset to a given stencil (factory may leave empty)
    pub fn StencilTable_GetOffsets(st: StencilTablePtr) -> IndexVectorRef;
    /// Returns the indices of the control vertices
    pub fn StencilTable_GetControlIndices(st: StencilTablePtr) -> IndexVectorRef;
    /// Returns the stencil interpolation weights
    pub fn StencilTable_GetWeights(st: StencilTablePtr) -> FloatVectorRef;
}
