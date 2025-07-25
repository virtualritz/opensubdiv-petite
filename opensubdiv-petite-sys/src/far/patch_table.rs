//! FFI bindings for OpenSubdiv Far::PatchTable and related types

use crate::far::TopologyRefiner;
use std::os::raw::{c_float, c_int};

/// Opaque type for Far::PatchTable
#[repr(C)]
pub struct PatchTable {
    _unused: [u8; 0],
}

/// Opaque type for Far::PatchTableFactory::Options
#[repr(C)]
pub struct PatchTableFactoryOptions {
    _unused: [u8; 0],
}

/// Opaque type for Far::PatchDescriptor
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PatchDescriptor {
    _data: [u8; 8], // Size of actual C++ PatchDescriptor
}

/// Opaque type for Far::PatchParam
#[repr(C)]
#[derive(Clone, Copy)]
pub struct PatchParam {
    _data: [u32; 3], // Size of actual C++ PatchParam
}

/// PatchDescriptor types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PatchDescriptorType {
    NonPatch = 0,
    Points = 1,
    Lines = 2,
    Quads = 3,
    Triangles = 4,
    Loop = 5,
    Regular = 6,
    BoundaryPattern0 = 7,
    BoundaryPattern1 = 8,
    BoundaryPattern2 = 9,
    BoundaryPattern3 = 10,
    BoundaryPattern4 = 11,
    CornerPattern0 = 12,
    CornerPattern1 = 13,
    CornerPattern2 = 14,
    CornerPattern3 = 15,
    CornerPattern4 = 16,
    Gregory = 17,
    GregoryBoundary = 18,
    GregoryCorner = 19,
    GregoryBasis = 20,
    GregoryTriangle = 21,
}

/// EndCap types for PatchTableFactory::Options
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EndCapType {
    None = 0,
    BSplineBasis = 1,
    GregoryBasis = 2,
    LegacyGregory = 3,
}

/// Triangle subdivision types
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TriangleSubdivision {
    Catmark = 0,
    Smooth = 1,
}

extern "C" {
    // PatchTableFactory functions
    pub fn PatchTableFactory_Create(
        refiner: *const TopologyRefiner,
        options: *const PatchTableFactoryOptions,
    ) -> *mut PatchTable;

    // PatchTable functions
    pub fn PatchTable_delete(table: *mut PatchTable);
    pub fn PatchTable_GetNumPatchArrays(table: *const PatchTable) -> c_int;
    pub fn PatchTable_GetNumPatches(table: *const PatchTable) -> c_int;
    pub fn PatchTable_GetNumControlVertices(table: *const PatchTable) -> c_int;
    pub fn PatchTable_GetMaxValence(table: *const PatchTable) -> c_int;
    pub fn PatchTable_GetNumPatches_PatchArray(table: *const PatchTable, array_index: c_int) -> c_int;
    pub fn PatchTable_GetPatchArrayDescriptor(
        table: *const PatchTable,
        array_index: c_int,
        desc: *mut PatchDescriptor,
    );
    pub fn PatchTable_GetPatchArrayVertices(
        table: *const PatchTable,
        array_index: c_int,
    ) -> *const c_int;
    pub fn PatchTable_GetPatchParam(
        table: *const PatchTable,
        array_index: c_int,
        patch_index: c_int,
        param: *mut PatchParam,
    );
    pub fn PatchTable_GetPatchControlVerticesTable(table: *const PatchTable) -> *const c_int;

    // PatchTableFactory::Options functions
    pub fn PatchTableFactory_Options_new() -> *mut PatchTableFactoryOptions;
    pub fn PatchTableFactory_Options_delete(options: *mut PatchTableFactoryOptions);
    pub fn PatchTableFactory_Options_SetEndCapType(options: *mut PatchTableFactoryOptions, end_cap_type: c_int);
    pub fn PatchTableFactory_Options_GetEndCapType(options: *const PatchTableFactoryOptions) -> c_int;
    pub fn PatchTableFactory_Options_SetTriangleSubdivision(
        options: *mut PatchTableFactoryOptions,
        triangle_subdivision: c_int,
    );
    pub fn PatchTableFactory_Options_SetUseInfSharpPatch(
        options: *mut PatchTableFactoryOptions,
        use_inf_sharp_patch: bool,
    );
    pub fn PatchTableFactory_Options_SetNumLegacyGregoryPatches(
        options: *mut PatchTableFactoryOptions,
        num_patches: c_int,
    );

    // PatchDescriptor functions
    pub fn PatchDescriptor_GetType(desc: *const PatchDescriptor) -> c_int;
    pub fn PatchDescriptor_GetNumControlVertices(desc: *const PatchDescriptor) -> c_int;
    pub fn PatchDescriptor_IsRegular(desc: *const PatchDescriptor) -> bool;

    // PatchParam functions
    pub fn PatchParam_GetUV(param: *const PatchParam, u: *mut c_float, v: *mut c_float);
    pub fn PatchParam_GetDepth(param: *const PatchParam) -> c_int;
    pub fn PatchParam_IsRegular(param: *const PatchParam) -> bool;
    pub fn PatchParam_GetBoundary(param: *const PatchParam) -> c_int;
    pub fn PatchParam_GetTransition(param: *const PatchParam) -> c_int;
}