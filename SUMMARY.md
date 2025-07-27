# OpenSubdiv-Petite Gregory Patch Support Summary

## Current Status

We've successfully added the missing C++ bindings for accessing patch local points, which fixed the InvalidControlPoints errors. However, we're still not generating Gregory patches for the cube mesh despite it having extraordinary vertices (valence 3).

## Completed Work

1. **Fixed Missing API Bindings**:
   - Added `PatchTable_GetNumLocalPoints()` and `PatchTable_GetLocalPointStencilTable()` C++ wrappers
   - Added corresponding FFI declarations in sys crate
   - Added Rust methods `local_point_count()` and `local_point_stencil_table()` to PatchTable
   - Created `StencilTableRef` type for borrowed stencil table references
   - Added `StencilTable_UpdateValues()` C++ wrapper with proper FloatValue wrapper class
   - Added `update_values()` method to StencilTableRef with idiomatic Rust API (returns Vec<f32>)

2. **Updated Gregory Patches Test**:
   - Test now computes local points using the stencil table
   - Vertex buffer correctly includes base vertices + refined vertices + local points
   - Fixed the issue where patch table expected 2688 control vertices but we only had 428
   - All patches now have valid control points (no more InvalidControlPoints errors)

3. **Debugging Findings**:
   - Cube mesh has 8 vertices, all with valence 3 (extraordinary vertices)
   - Despite this, patch table generates only Regular patches (168 total)
   - No GregoryBasis patches are generated (0 count)
   - Local points are computed correctly (168 local points added)
   - Final vertex buffer has 596 vertices (8 base + 420 refined + 168 local)

## Current Problem

The cube mesh is not generating Gregory patches despite having extraordinary vertices. The patch table creates 168 Regular patches instead of using Gregory patches at the extraordinary vertices. This results in holes in the STEP file output where Gregory patches should be.

## Investigation So Far

1. Verified that all 8 cube vertices have valence 3 (extraordinary)
2. Confirmed we're using `EndCapType::GregoryBasis` in patch options
3. Adaptive refinement is working (isolation level = 3)
4. Local points are being computed correctly via stencil table
5. Examples in OpenSubdiv codebase use similar setup with `ENDCAP_GREGORY_BASIS`

## Next Steps

1. **Investigate why Gregory patches aren't generated**:
   - Check if there are additional requirements for Gregory patch generation
   - Look at differences between our setup and working OpenSubdiv examples
   - May need to examine the patch builder logic in OpenSubdiv

2. **Implement trimmed NURBS approximation**:
   - Once we understand why patches are missing, implement fallback
   - Convert triangle patches to trimmed NURBS surfaces

## Test Output

Current test output shows:
```
Base level topology:
  Vertices: 8
  Faces: 6
  Edges: 12
  Vertex 0 has valence 3 (edges: [Index(0), Index(3), Index(4)])
  Vertex 1 has valence 3 (edges: [Index(0), Index(1), Index(5)])
  ... (all vertices have valence 3)

Patch counts:
  Regular: 168
  GregoryBasis: 0
  GregoryTriangle: 0
  Quads: 0
  Other: 0
```

## Files Modified

- `/opensubdiv-petite-sys/c-api/far/patch_table.cpp` - Added local point methods
- `/opensubdiv-petite-sys/c-api/far/stencil_table.cpp` - Added UpdateValues wrapper
- `/opensubdiv-petite-sys/src/far/patch_table.rs` - Added FFI declarations
- `/opensubdiv-petite-sys/src/far/stencil_table.rs` - Added FFI declaration
- `/opensubdiv-petite/src/far/patch_table.rs` - Added Rust API methods
- `/opensubdiv-petite/src/far/stencil_table.rs` - Added StencilTableRef and methods
- `/opensubdiv-petite/tests/gregory_patches.rs` - Updated test to use local points