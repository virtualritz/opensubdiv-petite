use crate::vtr::types::*;

/// Opaque pointer to a `LimitStencilTable`.
pub type LimitStencilTablePtr = *const std::ffi::c_void;

/// Flat FFI-safe replacement for C++ `LocationArray`.
#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LocationArrayDesc {
    pub ptex_idx: i32,
    pub num_locations: i32,
    pub s: *const f32,
    pub t: *const f32,
}

// The C++ struct uses bitfields packed into two unsigned ints:
// struct Options {
//     unsigned int interpolationMode           : 2,
//                  generate1stDerivatives      : 1,
//                  generate2ndDerivatives      : 1;
//     unsigned int fvarChannel;
// };

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct LimitStencilTableFactoryOptions {
    pub bitfield: u32,
    pub fvar_channel: u32,
}

impl Default for LimitStencilTableFactoryOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl LimitStencilTableFactoryOptions {
    pub fn new() -> Self {
        // Default: generate1stDerivatives = true (bit 2 set).
        Self {
            bitfield: 1 << 2,
            fvar_channel: 0,
        }
    }

    pub fn interpolation_mode(&self) -> u32 {
        self.bitfield & 0x3
    }

    pub fn set_interpolation_mode(&mut self, mode: u32) {
        self.bitfield = (self.bitfield & !0x3) | (mode & 0x3);
    }

    pub fn generate_1st_derivatives(&self) -> bool {
        (self.bitfield >> 2) & 0x1 == 1
    }

    pub fn set_generate_1st_derivatives(&mut self, value: bool) {
        if value {
            self.bitfield |= 1 << 2;
        } else {
            self.bitfield &= !(1 << 2);
        }
    }

    pub fn generate_2nd_derivatives(&self) -> bool {
        (self.bitfield >> 3) & 0x1 == 1
    }

    pub fn set_generate_2nd_derivatives(&mut self, value: bool) {
        if value {
            self.bitfield |= 1 << 3;
        } else {
            self.bitfield &= !(1 << 3);
        }
    }
}

#[link(name = "osd-capi", kind = "static")]
unsafe extern "C" {
    pub fn LimitStencilTable_destroy(table: LimitStencilTablePtr);

    pub fn LimitStencilTable_GetDuWeights(table: LimitStencilTablePtr) -> FloatVectorRef;
    pub fn LimitStencilTable_GetDvWeights(table: LimitStencilTablePtr) -> FloatVectorRef;
    pub fn LimitStencilTable_GetDuuWeights(table: LimitStencilTablePtr) -> FloatVectorRef;
    pub fn LimitStencilTable_GetDuvWeights(table: LimitStencilTablePtr) -> FloatVectorRef;
    pub fn LimitStencilTable_GetDvvWeights(table: LimitStencilTablePtr) -> FloatVectorRef;

    pub fn LimitStencilTableFactory_Create(
        refiner: *const crate::OpenSubdiv_v3_7_0_Far_TopologyRefiner,
        location_descs: *const LocationArrayDesc,
        num_arrays: i32,
        cv_stencils: *const std::ffi::c_void,
        patch_table: *const crate::far::patch_table::PatchTable,
        options_bitfield: u32,
        fvar_channel: u32,
    ) -> LimitStencilTablePtr;
}
