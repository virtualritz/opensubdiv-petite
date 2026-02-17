# BFR-Driven Coarse Export (Status)

## What changed

- Pulled OpenSubdiv subrepo to 3.7.0 (release branch).
- Added BFR C/Rust shims and helpers to build bicubic surfaces per base face without over-refining regular quads.
- Added mixed export: use BFR for regular faces, fall back to PatchTable for irregular (Gregory) patches.
- `test_cube_export.rs` now emits `cube_bfr.step` (BFR-only regulars) and `cube_bfr_mixed.step` (BFR regulars + PatchTable irregulars).
- New creased cube example (`cargo run --example creased_cube_export --no-default-features --features truck`) exercises sharp edges with BFR regular export, BFR+PatchTable mixed export, and stitched shell STEP output.

## Rationale

- BFR constructs per-face PatchTrees and limits refinement depth, so regular quad regions stay as single bicubic patches—fewer, larger patches closer to the control mesh.
- PatchTable still covers extraordinary regions; mixed path stitches both without over-subdividing everything.

## Key APIs

- `bfr_regular_surfaces(refiner, control_points, approx_smooth, approx_sharp)` → Vec<BSplineSurface> (regular faces only).
- `to_truck_surfaces_bfr_mixed(refiner, control_points, approx_smooth, approx_sharp)` → Vec<BSplineSurface> combining BFR regulars + PatchTable irregulars.

## Follow-ups

- Added creased cube example to exercise BFR mixed export with sharp edges.
- Consider extending BFR shim if we need irregular/Gregory support directly from BFR.
- Integrate mixed path into other exporters (OBJ/IGES) if desired.
- Stitched shell export shares vertices/edges across patches; useful for downstream boolean/STEP workflows (truck).
