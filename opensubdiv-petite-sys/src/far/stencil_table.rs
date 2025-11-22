use crate::vtr::types::*;

// The C++ struct uses bitfields packed into two unsigned ints:
// struct Options {
//     unsigned int interpolationMode           : 2,
//                  generateOffsets             : 1,
//                  generateControlVerts        : 1,
//                  generateIntermediateLevels  : 1,
//                  factorizeIntermediateLevels : 1,
//                  maxLevel                    : 4;
//     unsigned int fvarChannel;
// };

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StencilTableOptions {
    pub bitfield1: u32,    // Contains all the bit fields
    pub fvar_channel: u32, // Separate u32 for face-varying channel
}

impl Default for StencilTableOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl StencilTableOptions {
    pub fn new() -> Self {
        Self {
            bitfield1: 0,
            fvar_channel: 0,
        }
    }

    // Bit field accessors and setters
    pub fn interpolation_mode(&self) -> u32 {
        self.bitfield1 & 0x3 // 2 bits
    }

    pub fn set_interpolation_mode(&mut self, mode: u32) {
        self.bitfield1 = (self.bitfield1 & !0x3) | (mode & 0x3);
    }

    pub fn generate_offsets(&self) -> bool {
        (self.bitfield1 >> 2) & 0x1 == 1
    }

    pub fn set_generate_offsets(&mut self, value: bool) {
        if value {
            self.bitfield1 |= 1 << 2;
        } else {
            self.bitfield1 &= !(1 << 2);
        }
    }

    pub fn generate_control_vertices(&self) -> bool {
        (self.bitfield1 >> 3) & 0x1 == 1
    }

    pub fn set_generate_control_vertices(&mut self, value: bool) {
        if value {
            self.bitfield1 |= 1 << 3;
        } else {
            self.bitfield1 &= !(1 << 3);
        }
    }

    pub fn generate_intermediate_levels(&self) -> bool {
        (self.bitfield1 >> 4) & 0x1 == 1
    }

    pub fn set_generate_intermediate_levels(&mut self, value: bool) {
        if value {
            self.bitfield1 |= 1 << 4;
        } else {
            self.bitfield1 &= !(1 << 4);
        }
    }

    pub fn factorize_intermediate_levels(&self) -> bool {
        (self.bitfield1 >> 5) & 0x1 == 1
    }

    pub fn set_factorize_intermediate_levels(&mut self, value: bool) {
        if value {
            self.bitfield1 |= 1 << 5;
        } else {
            self.bitfield1 &= !(1 << 5);
        }
    }

    pub fn max_level(&self) -> u32 {
        (self.bitfield1 >> 6) & 0xF // 4 bits
    }

    pub fn set_max_level(&mut self, level: u32) {
        self.bitfield1 = (self.bitfield1 & !(0xF << 6)) | ((level & 0xF) << 6);
    }
}

pub type Stencil = crate::OpenSubdiv_v3_7_0_Far_Stencil;
pub type StencilTable = crate::OpenSubdiv_v3_7_0_Far_StencilTable;
pub type StencilTablePtr = *mut StencilTable;

#[link(name = "osd-capi", kind = "static")]
unsafe extern "C" {
    pub fn StencilTableFactory_Create(
        refiner: *mut crate::OpenSubdiv_v3_7_0_Far_TopologyRefiner,
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
    /// Update values by applying the stencil table
    pub fn StencilTable_UpdateValues(
        st: StencilTablePtr,
        src: *const f32,
        dst: *mut f32,
        start: i32,
        end: i32,
    );
}
