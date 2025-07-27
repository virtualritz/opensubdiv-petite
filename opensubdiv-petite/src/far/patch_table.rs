//! # Patch Table
//!
//! A `PatchTable` is a representation of the refined surface topology that can be
//! used for efficient evaluation of primvar data at arbitrary locations.
//!
//! The patches in a `PatchTable` are organized into patch arrays, where all patches
//! in a patch array have the same patch type. Each patch has a `PatchDescriptor`
//! that describes the number and arrangement of control points, and a `PatchParam`
//! that provides additional information about the patch's parameterization.

use super::StencilTableRef;
use crate::{Error, Index};
use opensubdiv_petite_sys as sys;
use std::marker::PhantomData;
use std::pin::Pin;

/// Options for creating a patch table
pub struct PatchTableOptions {
    inner: Pin<Box<sys::far::PatchTableFactoryOptions>>,
}

impl Default for PatchTableOptions {
    fn default() -> Self {
        Self::new()
    }
}

impl PatchTableOptions {
    /// Create a new PatchTableOptions with default settings
    pub fn new() -> Self {
        unsafe {
            let ptr = sys::far::PatchTableFactory_Options_new();
            assert!(!ptr.is_null());
            Self {
                inner: Pin::new(Box::from_raw(ptr)),
            }
        }
    }

    /// Set the end cap type
    pub fn end_cap_type(mut self, end_cap_type: EndCapType) -> Self {
        unsafe {
            sys::far::PatchTableFactory_Options_SetEndCapType(
                self.inner.as_mut().get_unchecked_mut(),
                end_cap_type as i32,
            );
        }
        self
    }

    /// Get the end cap type
    pub fn get_end_cap_type(&self) -> EndCapType {
        unsafe {
            let end_cap = sys::far::PatchTableFactory_Options_GetEndCapType(
                self.inner.as_ref().get_ref() as *const _,
            );
            match end_cap {
                0 => EndCapType::None,
                1 => EndCapType::BSplineBasis,
                2 => EndCapType::GregoryBasis,
                3 => EndCapType::LegacyGregory,
                _ => EndCapType::None,
            }
        }
    }

    /// Set the triangle subdivision type
    pub fn triangle_subdivision(mut self, triangle_subdivision: TriangleSubdivision) -> Self {
        unsafe {
            sys::far::PatchTableFactory_Options_SetTriangleSubdivision(
                self.inner.as_mut().get_unchecked_mut(),
                triangle_subdivision as i32,
            );
        }
        self
    }

    /// Set whether to use infinitely sharp patches
    pub fn use_inf_sharp_patch(mut self, use_inf_sharp_patch: bool) -> Self {
        unsafe {
            sys::far::PatchTableFactory_Options_SetUseInfSharpPatch(
                self.inner.as_mut().get_unchecked_mut(),
                use_inf_sharp_patch,
            );
        }
        self
    }

    /// Set the number of legacy Gregory patches
    pub fn num_legacy_gregory_patches(mut self, num_patches: i32) -> Self {
        unsafe {
            sys::far::PatchTableFactory_Options_SetNumLegacyGregoryPatches(
                self.inner.as_mut().get_unchecked_mut(),
                num_patches,
            );
        }
        self
    }

    pub(crate) fn as_ptr(&self) -> *const sys::far::PatchTableFactoryOptions {
        self.inner.as_ref().get_ref() as *const _
    }
}

impl Drop for PatchTableOptions {
    fn drop(&mut self) {
        unsafe {
            sys::far::PatchTableFactory_Options_delete(self.inner.as_mut().get_unchecked_mut());
        }
    }
}

/// End cap types for patch generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndCapType {
    /// No end caps
    None,
    /// B-spline basis end caps
    BSplineBasis,
    /// Gregory basis end caps
    GregoryBasis,
    /// Legacy Gregory end caps
    LegacyGregory,
}

// Re-export TriangleSubdivision from topology_refiner
pub use crate::far::topology_refiner::TriangleSubdivision;

/// A patch table containing refined surface patches
pub struct PatchTable {
    ptr: *mut sys::far::PatchTable,
    _phantom: PhantomData<sys::far::PatchTable>,
}

impl PatchTable {
    /// Create a new patch table from a topology refiner
    pub fn new(
        refiner: &crate::far::TopologyRefiner,
        options: Option<PatchTableOptions>,
    ) -> Result<Self, Error> {
        unsafe {
            let options_ptr = options
                .as_ref()
                .map(|o| o.as_ptr())
                .unwrap_or(std::ptr::null());

            let ptr = sys::far::PatchTableFactory_Create(refiner.as_ptr(), options_ptr);

            if ptr.is_null() {
                Err(Error::CreateTopologyRefinerFailed)
            } else {
                Ok(Self {
                    ptr,
                    _phantom: PhantomData,
                })
            }
        }
    }

    /// Get the number of patch arrays
    pub fn patch_arrays_len(&self) -> usize {
        unsafe { sys::far::PatchTable_GetNumPatchArrays(self.ptr) as usize }
    }

    /// Get the total number of patches
    pub fn patches_len(&self) -> usize {
        unsafe { sys::far::PatchTable_GetNumPatches(self.ptr) as usize }
    }

    /// Get the number of control vertices
    pub fn control_vertices_len(&self) -> usize {
        unsafe { sys::far::PatchTable_GetNumControlVertices(self.ptr) as usize }
    }

    /// Get the maximum valence
    pub fn max_valence(&self) -> usize {
        unsafe { sys::far::PatchTable_GetMaxValence(self.ptr) as usize }
    }

    /// Get the number of local points
    pub fn local_point_count(&self) -> usize {
        unsafe { sys::far::PatchTable_GetNumLocalPoints(self.ptr) as usize }
    }

    /// Get the stencil table for local points
    pub fn local_point_stencil_table(&self) -> Option<StencilTableRef<'_>> {
        unsafe {
            let stencil_ptr = sys::far::PatchTable_GetLocalPointStencilTable(self.ptr);
            if stencil_ptr.is_null() {
                None
            } else {
                Some(StencilTableRef {
                    ptr: stencil_ptr as *mut _,
                    _marker: std::marker::PhantomData,
                })
            }
        }
    }

    /// Get the number of patches in a specific patch array
    pub fn patch_array_patches_len(&self, array_index: usize) -> usize {
        unsafe {
            sys::far::PatchTable_GetNumPatches_PatchArray(self.ptr, array_index as i32) as usize
        }
    }

    /// Get the descriptor for a patch array
    pub fn patch_array_descriptor(&self, array_index: usize) -> Option<PatchDescriptor> {
        if array_index >= self.patch_arrays_len() {
            return None;
        }

        unsafe {
            let mut desc = std::mem::zeroed::<sys::far::PatchDescriptor>();
            sys::far::PatchTable_GetPatchArrayDescriptor(self.ptr, array_index as i32, &mut desc);
            Some(PatchDescriptor { inner: desc })
        }
    }

    /// Get the control vertex indices for a patch array
    pub fn patch_array_vertices(&self, array_index: usize) -> Option<&[Index]> {
        if array_index >= self.patch_arrays_len() {
            return None;
        }

        unsafe {
            let ptr = sys::far::PatchTable_GetPatchArrayVertices(self.ptr, array_index as i32);
            if ptr.is_null() {
                None
            } else {
                let len = self.patch_array_patches_len(array_index);
                let desc = self.patch_array_descriptor(array_index)?;
                let num_cvs = desc.control_vertices_len();
                let total_len = len * num_cvs;

                // Cast from i32 to Index (u32)
                Some(std::slice::from_raw_parts(ptr as *const Index, total_len))
            }
        }
    }

    /// Get the patch parameter for a specific patch
    pub fn patch_param(&self, array_index: usize, patch_index: usize) -> Option<PatchParam> {
        if array_index >= self.patch_arrays_len() {
            return None;
        }

        if patch_index >= self.patch_array_patches_len(array_index) {
            return None;
        }

        unsafe {
            let mut param = std::mem::zeroed::<sys::far::PatchParam>();
            sys::far::PatchTable_GetPatchParam(
                self.ptr,
                array_index as i32,
                patch_index as i32,
                &mut param,
            );
            Some(PatchParam { inner: param })
        }
    }

    /// Get all patch control vertex indices
    pub fn control_vertices_table(&self) -> Option<&[Index]> {
        unsafe {
            let ptr = sys::far::PatchTable_GetPatchControlVerticesTable(self.ptr);
            if ptr.is_null() {
                None
            } else {
                let len = self.control_vertices_len();
                // Cast from i32 to Index (u32)
                Some(std::slice::from_raw_parts(ptr as *const Index, len))
            }
        }
    }

    pub(crate) fn as_ptr(&self) -> *const sys::far::PatchTable {
        self.ptr
    }
}

impl Drop for PatchTable {
    fn drop(&mut self) {
        unsafe {
            sys::far::PatchTable_delete(self.ptr);
        }
    }
}

unsafe impl Send for PatchTable {}
unsafe impl Sync for PatchTable {}

/// Describes a patch type and its control point arrangement
#[derive(Clone, Copy)]
pub struct PatchDescriptor {
    inner: sys::far::PatchDescriptor,
}

impl PatchDescriptor {
    /// Get the patch type
    pub fn patch_type(&self) -> PatchType {
        unsafe {
            let patch_type = sys::far::PatchDescriptor_GetType(&self.inner);
            match patch_type {
                0 => PatchType::NonPatch,
                1 => PatchType::Points,
                2 => PatchType::Lines,
                3 => PatchType::Quads,
                4 => PatchType::Triangles,
                5 => PatchType::Loop,
                6 => PatchType::Regular,
                7 => PatchType::BoundaryPattern0,
                8 => PatchType::BoundaryPattern1,
                9 => PatchType::BoundaryPattern2,
                10 => PatchType::BoundaryPattern3,
                11 => PatchType::BoundaryPattern4,
                12 => PatchType::CornerPattern0,
                13 => PatchType::CornerPattern1,
                14 => PatchType::CornerPattern2,
                15 => PatchType::CornerPattern3,
                16 => PatchType::CornerPattern4,
                17 => PatchType::Gregory,
                18 => PatchType::GregoryBoundary,
                19 => PatchType::GregoryCorner,
                20 => PatchType::GregoryBasis,
                21 => PatchType::GregoryTriangle,
                _ => PatchType::NonPatch,
            }
        }
    }

    /// Get the number of control vertices for this patch type
    pub fn control_vertices_len(&self) -> usize {
        unsafe { sys::far::PatchDescriptor_GetNumControlVertices(&self.inner) as usize }
    }

    /// Check if this is a regular patch
    pub fn is_regular(&self) -> bool {
        unsafe { sys::far::PatchDescriptor_IsRegular(&self.inner) }
    }
}

/// Patch types supported by OpenSubdiv
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchType {
    /// Not a patch
    NonPatch,
    /// Point patches (1 control vertex)
    Points,
    /// Line patches (2 control vertices)
    Lines,
    /// Quad patches (4 control vertices)
    Quads,
    /// Triangle patches (3 control vertices)
    Triangles,
    /// Loop patches (12 control vertices)
    Loop,
    /// Regular patches (16 control vertices, bi-cubic B-spline)
    Regular,
    /// Boundary pattern patches
    BoundaryPattern0,
    BoundaryPattern1,
    BoundaryPattern2,
    BoundaryPattern3,
    BoundaryPattern4,
    /// Corner pattern patches
    CornerPattern0,
    CornerPattern1,
    CornerPattern2,
    CornerPattern3,
    CornerPattern4,
    /// Gregory patches
    Gregory,
    GregoryBoundary,
    GregoryCorner,
    GregoryBasis,
    GregoryTriangle,
}

/// Parameters for a patch
#[derive(Clone, Copy)]
pub struct PatchParam {
    inner: sys::far::PatchParam,
}

impl PatchParam {
    /// Get the UV coordinates of the patch
    pub fn uv(&self) -> (f32, f32) {
        unsafe {
            let mut u = 0.0;
            let mut v = 0.0;
            sys::far::PatchParam_GetUV(&self.inner, &mut u, &mut v);
            (u, v)
        }
    }

    /// Get the subdivision depth of the patch
    pub fn depth(&self) -> usize {
        unsafe { sys::far::PatchParam_GetDepth(&self.inner) as usize }
    }

    /// Check if this is a regular patch
    pub fn is_regular(&self) -> bool {
        unsafe { sys::far::PatchParam_IsRegular(&self.inner) }
    }

    /// Get the boundary mask
    pub fn boundary(&self) -> i32 {
        unsafe { sys::far::PatchParam_GetBoundary(&self.inner) }
    }

    /// Get the transition mask
    pub fn transition(&self) -> i32 {
        unsafe { sys::far::PatchParam_GetTransition(&self.inner) }
    }
}

/// Result of patch evaluation containing point and derivatives
#[derive(Clone, Copy)]
pub struct PatchEvalResult {
    /// Evaluated point position
    pub point: [f32; 3],
    /// First derivative with respect to u
    pub du: [f32; 3],
    /// First derivative with respect to v
    pub dv: [f32; 3],
    /// Second derivative with respect to u
    pub duu: [f32; 3],
    /// Mixed second derivative
    pub duv: [f32; 3],
    /// Second derivative with respect to v
    pub dvv: [f32; 3],
}

impl From<sys::far::PatchEvalResult> for PatchEvalResult {
    fn from(result: sys::far::PatchEvalResult) -> Self {
        Self {
            point: result.point,
            du: result.du,
            dv: result.dv,
            duu: result.duu,
            duv: result.duv,
            dvv: result.dvv,
        }
    }
}

/// Result of basis evaluation containing weights for position and derivatives
pub type BasisWeights = (Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>, Vec<f32>);

impl PatchTable {
    /// Evaluate basis functions for a patch at given parametric coordinates
    pub fn evaluate_basis(&self, patch_index: usize, u: f32, v: f32) -> Option<BasisWeights> {
        if patch_index >= self.patches_len() {
            return None;
        }

        // Find which patch array this patch belongs to
        let mut array_index = 0;
        let mut local_patch_index = patch_index;

        for i in 0..self.patch_arrays_len() {
            let num_patches = self.patch_array_patches_len(i);
            if local_patch_index < num_patches {
                array_index = i;
                break;
            }
            local_patch_index -= num_patches;
        }

        // Get the number of control vertices for this patch
        let desc = self.patch_array_descriptor(array_index)?;
        let num_cvs = desc.control_vertices_len();

        // Allocate vectors for weights
        let mut w_p = vec![0.0f32; num_cvs];
        let mut w_du = vec![0.0f32; num_cvs];
        let mut w_dv = vec![0.0f32; num_cvs];
        let mut w_duu = vec![0.0f32; num_cvs];
        let mut w_duv = vec![0.0f32; num_cvs];
        let mut w_dvv = vec![0.0f32; num_cvs];

        unsafe {
            let success = sys::far::PatchTable_EvaluateBasis(
                self.ptr,
                patch_index as i32,
                u,
                v,
                w_p.as_mut_ptr(),
                w_du.as_mut_ptr(),
                w_dv.as_mut_ptr(),
                w_duu.as_mut_ptr(),
                w_duv.as_mut_ptr(),
                w_dvv.as_mut_ptr(),
            );

            if success {
                Some((w_p, w_du, w_dv, w_duu, w_duv, w_dvv))
            } else {
                None
            }
        }
    }

    /// Evaluate a patch at given parametric coordinates using control points
    pub fn evaluate_point(
        &self,
        patch_index: usize,
        u: f32,
        v: f32,
        control_points: &[[f32; 3]],
    ) -> Option<PatchEvalResult> {
        if patch_index >= self.patches_len() {
            return None;
        }

        unsafe {
            let mut result = std::mem::zeroed::<sys::far::PatchEvalResult>();

            let success = sys::far::PatchTable_EvaluatePoint(
                self.ptr,
                patch_index as i32,
                u,
                v,
                control_points.as_ptr() as *const f32,
                control_points.len() as i32,
                &mut result,
            );

            if success {
                Some(result.into())
            } else {
                None
            }
        }
    }
}

/// Map for efficient patch location from face coordinates
pub struct PatchMap {
    ptr: *mut sys::far::PatchMap,
    _phantom: PhantomData<sys::far::PatchMap>,
}

impl PatchMap {
    /// Create a new patch map from a patch table
    pub fn new(patch_table: &PatchTable) -> Option<Self> {
        unsafe {
            let ptr = sys::far::PatchMap_Create(patch_table.as_ptr());
            if ptr.is_null() {
                None
            } else {
                Some(Self {
                    ptr,
                    _phantom: PhantomData,
                })
            }
        }
    }

    /// Find the patch containing a given face at parametric coordinates
    pub fn find_patch(&self, face_index: usize, u: f32, v: f32) -> Option<(usize, f32, f32)> {
        unsafe {
            let mut patch_index = 0i32;
            let mut patch_u = 0.0f32;
            let mut patch_v = 0.0f32;

            let found = sys::far::PatchMap_FindPatch(
                self.ptr,
                face_index as i32,
                u,
                v,
                &mut patch_index,
                &mut patch_u,
                &mut patch_v,
            );

            if found {
                Some((patch_index as usize, patch_u, patch_v))
            } else {
                None
            }
        }
    }
}

impl Drop for PatchMap {
    fn drop(&mut self) {
        unsafe {
            sys::far::PatchMap_delete(self.ptr);
        }
    }
}

unsafe impl Send for PatchMap {}
unsafe impl Sync for PatchMap {}
