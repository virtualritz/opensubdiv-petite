//! Limit stencil table for evaluation at arbitrary parametric locations.
//!
//! A [`LimitStencilTable`] extends [`StencilTable`] with
//! derivative weights (du, dv, and optionally duu, duv, dvv) so that tangent
//! vectors and curvature can be evaluated efficiently on the limit surface.

use opensubdiv_petite_sys as sys;

use crate::far::stencil_table::InterpolationMode;
use crate::far::{PatchTable, StencilTable, TopologyRefiner};
use crate::Index;

/// Describes a set of sample locations on a single ptex face.
#[derive(Debug, Clone)]
pub struct LocationArray<'a> {
    /// Ptex face index.
    pub ptex_index: usize,
    /// Parametric u coordinates.
    pub s: &'a [f32],
    /// Parametric v coordinates.
    pub t: &'a [f32],
}

/// Options for creating a [`LimitStencilTable`].
#[derive(Debug, Clone)]
pub struct LimitStencilTableOptions {
    /// Interpolation mode (vertex, varying, or face-varying).
    pub interpolation_mode: InterpolationMode,
    /// Generate 1st derivative weights (du, dv). Default: `true`.
    pub generate_1st_derivatives: bool,
    /// Generate 2nd derivative weights (duu, duv, dvv). Default: `false`.
    pub generate_2nd_derivatives: bool,
    /// Face-varying channel index.
    pub face_varying_channel: usize,
}

impl Default for LimitStencilTableOptions {
    fn default() -> Self {
        Self {
            interpolation_mode: InterpolationMode::Vertex,
            generate_1st_derivatives: true,
            generate_2nd_derivatives: false,
            face_varying_channel: 0,
        }
    }
}

/// Table of limit stencils with derivative weights.
///
/// Created via [`LimitStencilTable::new`] from a [`TopologyRefiner`] and a set
/// of parametric sample locations. Inherits base stencil data (sizes, offsets,
/// indices, weights) from the C++ `StencilTable` base class and adds five
/// derivative weight vectors.
pub struct LimitStencilTable {
    // AIDEV-NOTE: This pointer is to a C++ `LimitStencilTable` which inherits
    // from `StencilTable`. For base-class accessor calls we cast it to
    // `StencilTablePtr` via `as_base_ptr()`. This is safe because the C++
    // object layout guarantees single-inheritance pointer identity.
    ptr: sys::far::LimitStencilTablePtr,
    has_1st_derivs: bool,
    has_2nd_derivs: bool,
}

impl Drop for LimitStencilTable {
    #[inline]
    fn drop(&mut self) {
        unsafe { sys::far::limit_stencil_table::LimitStencilTable_destroy(self.ptr) }
    }
}

// AIDEV-NOTE: Safe because the underlying C++ table is immutable after
// creation.
unsafe impl Send for LimitStencilTable {}
unsafe impl Sync for LimitStencilTable {}

impl LimitStencilTable {
    /// Create a limit stencil table from a topology refiner and sample
    /// locations.
    ///
    /// Each [`LocationArray`] specifies a ptex face and a set of (s, t)
    /// coordinates on that face. The `s` and `t` slices must have equal length.
    ///
    /// `cv_stencils` and `patch_table` are optional and mirror the C++ factory
    /// overloads. Pass `None` to let the factory build them internally.
    pub fn new(
        refiner: &TopologyRefiner,
        locations: &[LocationArray<'_>],
        cv_stencils: Option<&StencilTable>,
        patch_table: Option<&PatchTable>,
        options: LimitStencilTableOptions,
    ) -> crate::Result<Self> {
        // Validate that s and t slices match in each location array.
        for loc in locations {
            if loc.s.len() != loc.t.len() {
                return Err(crate::Error::InvalidTopology(format!(
                    "LocationArray for ptex face {}: s.len()={} != t.len()={}",
                    loc.ptex_index,
                    loc.s.len(),
                    loc.t.len()
                )));
            }
        }

        let ffi_descs: Vec<sys::far::limit_stencil_table::LocationArrayDesc> = locations
            .iter()
            .map(|loc| sys::far::limit_stencil_table::LocationArrayDesc {
                ptex_idx: loc.ptex_index as i32,
                num_locations: loc.s.len() as i32,
                s: loc.s.as_ptr(),
                t: loc.t.as_ptr(),
            })
            .collect();

        let mut opts = sys::far::limit_stencil_table::LimitStencilTableFactoryOptions::new();
        opts.set_interpolation_mode(options.interpolation_mode as u32);
        opts.set_generate_1st_derivatives(options.generate_1st_derivatives);
        opts.set_generate_2nd_derivatives(options.generate_2nd_derivatives);
        opts.fvar_channel = options.face_varying_channel as u32;

        let cv_ptr = cv_stencils
            .map(|s| s.0 as *const std::ffi::c_void)
            .unwrap_or(std::ptr::null());

        let patch_ptr = patch_table.map(|p| p.as_ptr()).unwrap_or(std::ptr::null());

        let ptr = unsafe {
            sys::far::limit_stencil_table::LimitStencilTableFactory_Create(
                refiner.as_ptr() as *const _,
                ffi_descs.as_ptr(),
                ffi_descs.len() as i32,
                cv_ptr,
                patch_ptr,
                opts.bitfield,
                opts.fvar_channel,
            )
        };

        if ptr.is_null() {
            return Err(crate::Error::StencilTableCreation);
        }

        Ok(Self {
            ptr,
            has_1st_derivs: options.generate_1st_derivatives,
            has_2nd_derivs: options.generate_2nd_derivatives,
        })
    }

    /// Cast to base `StencilTablePtr` for base-class FFI accessors.
    #[inline]
    fn as_base_ptr(&self) -> sys::far::StencilTablePtr {
        self.ptr as sys::far::StencilTablePtr
    }

    /// Returns the number of stencils in the table.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { sys::far::stencil_table::StencilTable_GetNumStencils(self.as_base_ptr()) as _ }
    }

    /// Returns `true` if the table is empty.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the number of control vertices indexed in the table.
    #[inline]
    pub fn control_vertex_count(&self) -> usize {
        unsafe {
            sys::far::stencil_table::StencilTable_GetNumControlVertices(self.as_base_ptr()) as _
        }
    }

    /// Returns the number of control vertices of each stencil in the table.
    #[inline]
    pub fn sizes(&self) -> &[i32] {
        let vr = unsafe { sys::far::stencil_table::StencilTable_GetSizes(self.as_base_ptr()) };
        if vr.data().is_null() || vr.size() == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(vr.data() as _, vr.size()) }
        }
    }

    /// Returns the offset to a given stencil.
    #[inline]
    pub fn offsets(&self) -> &[Index] {
        let vr = unsafe { sys::far::stencil_table::StencilTable_GetOffsets(self.as_base_ptr()) };
        if vr.data().is_null() || vr.size() == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(vr.data() as *const Index, vr.size()) }
        }
    }

    /// Returns the indices of the control vertices.
    #[inline]
    pub fn control_indices(&self) -> &[Index] {
        let vr =
            unsafe { sys::far::stencil_table::StencilTable_GetControlIndices(self.as_base_ptr()) };
        if vr.data().is_null() || vr.size() == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(vr.data() as *const Index, vr.size()) }
        }
    }

    /// Returns the stencil interpolation weights.
    #[inline]
    pub fn weights(&self) -> &[f32] {
        let vr = unsafe { sys::far::stencil_table::StencilTable_GetWeights(self.as_base_ptr()) };
        if vr.data().is_null() || vr.size() == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(vr.data(), vr.size()) }
        }
    }

    /// Returns the du derivative weights.
    #[inline]
    pub fn du_weights(&self) -> &[f32] {
        let vr = unsafe { sys::far::limit_stencil_table::LimitStencilTable_GetDuWeights(self.ptr) };
        if vr.data().is_null() || vr.size() == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(vr.data(), vr.size()) }
        }
    }

    /// Returns the dv derivative weights.
    #[inline]
    pub fn dv_weights(&self) -> &[f32] {
        let vr = unsafe { sys::far::limit_stencil_table::LimitStencilTable_GetDvWeights(self.ptr) };
        if vr.data().is_null() || vr.size() == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(vr.data(), vr.size()) }
        }
    }

    /// Returns the duu derivative weights.
    #[inline]
    pub fn duu_weights(&self) -> &[f32] {
        let vr =
            unsafe { sys::far::limit_stencil_table::LimitStencilTable_GetDuuWeights(self.ptr) };
        if vr.data().is_null() || vr.size() == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(vr.data(), vr.size()) }
        }
    }

    /// Returns the duv derivative weights.
    #[inline]
    pub fn duv_weights(&self) -> &[f32] {
        let vr =
            unsafe { sys::far::limit_stencil_table::LimitStencilTable_GetDuvWeights(self.ptr) };
        if vr.data().is_null() || vr.size() == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(vr.data(), vr.size()) }
        }
    }

    /// Returns the dvv derivative weights.
    #[inline]
    pub fn dvv_weights(&self) -> &[f32] {
        let vr =
            unsafe { sys::far::limit_stencil_table::LimitStencilTable_GetDvvWeights(self.ptr) };
        if vr.data().is_null() || vr.size() == 0 {
            &[]
        } else {
            unsafe { std::slice::from_raw_parts(vr.data(), vr.size()) }
        }
    }

    /// Whether 1st derivative weights (du, dv) were generated.
    #[inline]
    pub fn has_1st_derivatives(&self) -> bool {
        self.has_1st_derivs
    }

    /// Whether 2nd derivative weights (duu, duv, dvv) were generated.
    #[inline]
    pub fn has_2nd_derivatives(&self) -> bool {
        self.has_2nd_derivs
    }
}

impl std::fmt::Debug for LimitStencilTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LimitStencilTable")
            .field("len", &self.len())
            .field("control_vertex_count", &self.control_vertex_count())
            .field("has_1st_derivatives", &self.has_1st_derivs)
            .field("has_2nd_derivatives", &self.has_2nd_derivs)
            .finish()
    }
}
