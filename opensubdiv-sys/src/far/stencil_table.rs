use crate::vtr::types::*;

#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum InterpolationMode {
    Vertex = 0, // FIXME: bindgen curre
    Varying,
    FaceVarying,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct Options {
    interpolation_mode: u32,
    generate_offsets: u32,
    generate_control_vertices: u32,
    generate_intermediate_levels: u32,
    factorize_intermediate_levels: u32,
    max_level: u32,
    face_varying_channel: u32,
}

impl Default for Options {
    fn default() -> Self {
        Self {
            interpolation_mode: InterpolationMode::Vertex as _,
            generate_offsets: false as _,
            generate_control_vertices: false as _,
            generate_intermediate_levels: true as _,
            factorize_intermediate_levels: true as _,
            max_level: 10,
            face_varying_channel: 0,
        }
    }
}

impl Options {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn interpolation_mode(
        &mut self,
        interpolation_mode: InterpolationMode,
    ) -> &mut Self {
        self.interpolation_mode = interpolation_mode as _;
        self
    }

    pub fn generate_offsets(&mut self, generate_offsets: bool) -> &mut Self {
        self.generate_offsets = generate_offsets as _;
        self
    }

    pub fn generate_control_vertices(
        &mut self,
        generate_control_vertices: bool,
    ) -> &mut Self {
        self.generate_control_vertices = generate_control_vertices as _;
        self
    }

    pub fn generate_intermediate_levels(
        &mut self,
        generate_intermediate_levels: bool,
    ) -> &mut Self {
        self.generate_intermediate_levels = generate_intermediate_levels as _;
        self
    }

    pub fn factorize_intermediate_levels(
        &mut self,
        factorize_intermediate_levels: bool,
    ) -> &mut Self {
        self.factorize_intermediate_levels = factorize_intermediate_levels as _;
        self
    }

    pub fn max_level(&mut self, max_level: u32) -> &mut Self {
        self.max_level = max_level;
        self
    }
}

pub type Stencil = crate::OpenSubdiv_v3_4_4_Far_StencilReal<f32>;
pub type StencilTable = crate::OpenSubdiv_v3_4_4_Far_StencilTableReal;
pub type StencilTablePtr = *mut StencilTable;

extern "C" {
    pub fn StencilTableFactory_Create(
        refiner: *mut crate::OpenSubdiv_v3_4_4_Far_TopologyRefiner,
        options: Options,
    ) -> StencilTablePtr;

    pub fn StencilTable_destroy(st: StencilTablePtr);
    /// Returns the number of stencils in the table
    pub fn StencilTable_GetNumStencils(st: StencilTablePtr) -> u32;
    /// Returns the number of control vertices indexed in the table
    pub fn StencilTable_GetNumControlVertices(st: StencilTablePtr) -> u32;
    /// Returns a Stencil at index i in the table
    pub fn StencilTable_GetStencil(
        st: StencilTablePtr,
        index: Index,
    ) -> Stencil;
    /// Returns the number of control vertices of each stencil in the table
    pub fn StencilTable_GetSizes(st: StencilTablePtr) -> IntVectorRef;
    /// Returns the offset to a given stencil (factory may leave empty)
    pub fn StencilTable_GetOffsets(st: StencilTablePtr) -> IndexVectorRef;
    /// Returns the indices of the control vertices
    pub fn StencilTable_GetControlIndices(
        st: StencilTablePtr,
    ) -> IndexVectorRef;
    /// Returns the stencil interpolation weights
    pub fn StencilTable_GetWeights(st: StencilTablePtr) -> FloatVectorRef;
}
