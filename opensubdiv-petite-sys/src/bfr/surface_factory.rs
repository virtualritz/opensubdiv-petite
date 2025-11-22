#![allow(non_camel_case_types)]

use crate::far::topology_refiner::TopologyRefinerPtr;

#[repr(C)]
pub struct Bfr_SurfaceFactory_f {
    _private: [u8; 0],
}

#[repr(C)]
pub struct Bfr_Surface_f {
    _private: [u8; 0],
}

#[link(name = "osd-capi", kind = "static")]
unsafe extern "C" {
    pub fn Bfr_SurfaceFactory_Create(
        refiner: TopologyRefinerPtr,
        approx_level_smooth: ::std::os::raw::c_int,
        approx_level_sharp: ::std::os::raw::c_int,
    ) -> *mut Bfr_SurfaceFactory_f;

    pub fn Bfr_SurfaceFactory_Destroy(factory: *mut Bfr_SurfaceFactory_f);

    pub fn Bfr_Surface_Create() -> *mut Bfr_Surface_f;
    pub fn Bfr_Surface_Destroy(surface: *mut Bfr_Surface_f);

    pub fn Bfr_SurfaceFactory_InitVertexSurface(
        factory: *const Bfr_SurfaceFactory_f,
        face_index: ::std::os::raw::c_int,
        surface: *mut Bfr_Surface_f,
    ) -> bool;

    pub fn Bfr_Surface_IsValid(surface: *const Bfr_Surface_f) -> bool;

    pub fn Bfr_Surface_IsRegular(surface: *const Bfr_Surface_f) -> bool;

    pub fn Bfr_Surface_GetNumControlPoints(surface: *const Bfr_Surface_f) -> ::std::os::raw::c_int;

    pub fn Bfr_Surface_GetControlPointIndices(
        surface: *const Bfr_Surface_f,
        out_indices: *mut ::std::os::raw::c_int,
        max_count: ::std::os::raw::c_int,
    ) -> ::std::os::raw::c_int;

    pub fn Bfr_Surface_GetNumPatchPoints(surface: *const Bfr_Surface_f) -> ::std::os::raw::c_int;

    pub fn Bfr_Surface_GatherPatchPoints(
        surface: *const Bfr_Surface_f,
        mesh_points: *const f32,
        mesh_stride: ::std::os::raw::c_int,
        out_patch_points: *mut f32,
        max_points: ::std::os::raw::c_int,
    ) -> bool;

    pub fn Bfr_Surface_EvaluatePosition(
        surface: *const Bfr_Surface_f,
        u: f32,
        v: f32,
        mesh_points: *const f32,
        mesh_stride: ::std::os::raw::c_int,
        out_p3: *mut f32,
    ) -> bool;
}
