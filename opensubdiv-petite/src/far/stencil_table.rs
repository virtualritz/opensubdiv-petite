//! Table of subdivision stencils.
//!
//! Stencils are the most direct method of evaluation of locations on the limit
//! of a surface. Every point of a limit surface can be computed by linearly
//! blending a collection of coarse control vertices.
//!
//! A stencil assigns a series of control vertex indices with a blending weight
//1 that corresponds to a unique parametric location of the limit surface. When
//! the control vertices move in space, the limit location can be very
//! efficiently recomputed simply by applying the blending weights to the
//! series of coarse control vertices.
use opensubdiv_petite_sys as sys;
use std::convert::TryInto;

use crate::Index;

use crate::far::TopologyRefiner;

/// Gives read access to a single stencil in a [`StencilTable`].
pub struct Stencil<'a> {
    indices: &'a [Index],
    weights: &'a [f32],
}

impl<'a> Stencil<'a> {
    /// Returns the indices of the control vertices.
    pub fn indices(&self) -> &'a [Index] {
        self.indices
    }

    /// Returns the stencil interpolation weights.
    pub fn weights(&self) -> &'a [f32] {
        self.weights
    }
}

/// Container for stencil data.
pub struct StencilTable(pub(crate) sys::far::StencilTablePtr);

/// Borrowed reference to a stencil table.
pub struct StencilTableRef<'a> {
    pub(crate) ptr: sys::far::StencilTablePtr,
    pub(crate) _marker: std::marker::PhantomData<&'a ()>,
}

impl Drop for StencilTable {
    #[inline]
    fn drop(&mut self) {
        unsafe { sys::far::stencil_table::StencilTable_destroy(self.0) }
    }
}

impl StencilTable {
    /// Create a new stencil table.
    pub fn new(refiner: &TopologyRefiner, options: StencilTableOptions) -> StencilTable {
        let mut sys_options = sys::far::stencil_table::StencilTableOptions::new();

        // Set the bitfield values
        sys_options.set_interpolation_mode(options.interpolation_mode as u32);
        sys_options.set_generate_offsets(options.generate_offsets);
        sys_options.set_generate_control_vertices(options.generate_control_vertices);
        sys_options.set_generate_intermediate_levels(options.generate_intermediate_levels);
        sys_options.set_factorize_intermediate_levels(options.factorize_intermediate_levels);
        sys_options.set_max_level(options.max_level.try_into().unwrap());
        sys_options.fvar_channel = options.face_varying_channel.try_into().unwrap();

        let ptr =
            unsafe { sys::far::stencil_table::StencilTableFactory_Create(refiner.0, sys_options) };

        if ptr.is_null() {
            panic!("StencilTableFactory_Create() returned null");
        }

        StencilTable(ptr)
    }

    /// Returns the number of stencils in the table.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { sys::far::stencil_table::StencilTable_GetNumStencils(self.0) as _ }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        0 == self.len()
    }

    /// Returns the number of control vertices indexed in the table.
    #[inline]
    pub fn control_vertex_count(&self) -> usize {
        unsafe { sys::far::stencil_table::StencilTable_GetNumControlVertices(self.0) as _ }
    }

    /// Returns the number of control vertices indexed in the table.
    #[deprecated(since = "0.3.0", note = "Use `control_vertex_count` instead")]
    #[inline]
    pub fn control_vertices_len(&self) -> usize {
        self.control_vertex_count()
    }

    /// Returns a Stencil at index i in the table.
    #[inline]
    pub fn stencil(&self, i: Index) -> Option<Stencil<'_>> {
        if self.len() <= i.into() {
            None
        } else {
            unsafe {
                let stencil = sys::far::stencil_table::StencilTable_GetStencil(self.0, i.into());
                Some(Stencil {
                    indices: std::slice::from_raw_parts(
                        stencil._base._indices as _,
                        *stencil._base._size as _,
                    ),
                    weights: std::slice::from_raw_parts(
                        stencil._base._weights,
                        *stencil._base._size as _,
                    ),
                })
            }
        }
    }

    /// Returns the number of control vertices of each stencil in the table.
    #[inline]
    pub fn sizes(&self) -> &[i32] {
        unsafe {
            let vr = sys::far::stencil_table::StencilTable_GetSizes(self.0);
            std::slice::from_raw_parts(vr.data() as _, vr.size())
        }
    }

    /// Returns the offset to a given stencil (factory may leave empty).
    #[inline]
    pub fn offsets(&self) -> &[Index] {
        unsafe {
            let vr = sys::far::stencil_table::StencilTable_GetOffsets(self.0);
            std::slice::from_raw_parts(vr.data() as *const Index, vr.size())
        }
    }

    /// Returns the indices of the control vertices.
    #[inline]
    pub fn control_indices(&self) -> &[Index] {
        unsafe {
            let vr = sys::far::stencil_table::StencilTable_GetControlIndices(self.0);
            std::slice::from_raw_parts(vr.data() as *const Index, vr.size())
        }
    }

    /// Returns the stencil interpolation weights.
    #[inline]
    pub fn weights(&self) -> &[f32] {
        unsafe {
            let vr = sys::far::stencil_table::StencilTable_GetWeights(self.0);
            std::slice::from_raw_parts(vr.data(), vr.size())
        }
    }

    /// Update values by applying the stencil table
    ///
    /// # Arguments
    /// * `src` - Source values to interpolate from
    /// * `start` - Optional index of first destination value to update
    /// * `end` - Optional index of last destination value to update
    ///
    /// # Returns
    /// A vector containing the interpolated values
    pub fn update_values(&self, src: &[f32], start: Option<usize>, end: Option<usize>) -> Vec<f32> {
        self.update_values_impl(self.0, src, start, end)
    }

    fn update_values_impl(
        &self,
        ptr: sys::far::StencilTablePtr,
        src: &[f32],
        start: Option<usize>,
        end: Option<usize>,
    ) -> Vec<f32> {
        // Determine the output size based on the number of stencils
        let num_stencils =
            unsafe { sys::far::stencil_table::StencilTable_GetNumStencils(ptr) as usize };
        let actual_start = start.unwrap_or(0);
        let actual_end = end.unwrap_or(num_stencils);
        let output_size = actual_end - actual_start;

        // AIDEV-NOTE: Local point stencil tables may report 0 control vertices
        // In this case, we assume the source array size matches what's expected
        // The output will have one value per stencil

        // Create output buffer with the size matching number of stencils
        let mut dst = Vec::with_capacity(output_size);

        unsafe {
            sys::far::stencil_table::StencilTable_UpdateValues(
                ptr,
                src.as_ptr(),
                dst.as_mut_ptr(),
                start.map(|s| s as i32).unwrap_or(-1),
                end.map(|e| e as i32).unwrap_or(-1),
            );

            // Set length after successful update
            dst.set_len(output_size);
        }

        dst
    }
}

//
#[repr(u32)]
#[derive(Clone, Copy, Debug)]
pub enum InterpolationMode {
    Vertex = 0,
    Varying,
    FaceVarying,
}

#[repr(C)]
#[derive(Clone, Copy, Debug)]
pub struct StencilTableOptions {
    pub interpolation_mode: InterpolationMode,
    pub generate_offsets: bool,
    pub generate_control_vertices: bool,
    pub generate_intermediate_levels: bool,
    pub factorize_intermediate_levels: bool,
    pub max_level: usize,
    pub face_varying_channel: usize,
}

impl<'a> StencilTableRef<'a> {
    /// Returns the number of stencils in the table.
    #[inline]
    pub fn len(&self) -> usize {
        unsafe { sys::far::stencil_table::StencilTable_GetNumStencils(self.ptr) as _ }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        0 == self.len()
    }

    /// Returns the number of control vertices indexed in the table.
    #[inline]
    pub fn control_vertex_count(&self) -> usize {
        unsafe { sys::far::stencil_table::StencilTable_GetNumControlVertices(self.ptr) as _ }
    }

    /// Update values by applying the stencil table
    pub fn update_values(&self, src: &[f32], start: Option<usize>, end: Option<usize>) -> Vec<f32> {
        // Use the same implementation as StencilTable
        StencilTable(std::ptr::null_mut()).update_values_impl(self.ptr, src, start, end)
    }
}

impl Default for StencilTableOptions {
    fn default() -> Self {
        Self {
            interpolation_mode: InterpolationMode::Vertex,
            generate_offsets: false,
            generate_control_vertices: false,
            generate_intermediate_levels: true,
            factorize_intermediate_levels: true,
            max_level: 10,
            face_varying_channel: 0,
        }
    }
}
