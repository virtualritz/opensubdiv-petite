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

impl Drop for StencilTable {
    #[inline]
    fn drop(&mut self) {
        unsafe { sys::far::stencil_table::StencilTable_destroy(self.0) }
    }
}

impl StencilTable {
    /// Create a new stencil table.
    pub fn new(refiner: &TopologyRefiner, options: StencilTableOptions) -> StencilTable {
        let ptr = unsafe {
            sys::far::stencil_table::StencilTableFactory_Create(
                refiner.0,
                sys::far::stencil_table::StencilTableOptions {
                    interpolation_mode: options.interpolation_mode as _,
                    generate_offsets: options.generate_offsets as _,
                    generate_control_vertices: options.generate_control_vertices as _,
                    generate_intermediate_levels: options.generate_intermediate_levels as _,
                    factorize_intermediate_levels: options.factorize_intermediate_levels as _,
                    max_level: options.max_level.try_into().unwrap(),
                    face_varying_channel: options.face_varying_channel.try_into().unwrap(),
                },
            )
        };

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
                    indices: std::slice::from_raw_parts(stencil._indices as _, stencil._size as _),
                    weights: std::slice::from_raw_parts(stencil._weights, stencil._size as _),
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
