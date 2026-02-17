# Active Context

## Current Work Focus

### wgpu Compute Backend (In Progress)

- **`src/osd/wgpu.rs`** -- WGSL compute evaluator for GPU stencil evaluation via wgpu.
- **`shaders/wgsl/stencil_eval.wgsl`** -- Compute kernel translated from OpenSubdiv's GLSL kernel.
- **`tests/wgpu_stencil.rs`** -- CPU vs GPU parity test.
- **Status:** Position evaluation works. Derivatives NOT yet wired (AIDEV-TODO at wgpu.rs:475).
- **Key constraint:** `StencilTableOptions::generate_offsets` must be `true` for GPU path.

### truck CAD Integration (Heavily Active)

- **`src/truck.rs`** (~2500+ lines) -- Most active development area.
- Recent additions: `GregoryAccuracy`, `StepExportOptions`, superpatch merging, BFR mixed-mode surfaces, gap filling, boundary control point fixes.
- Multiple export strategies: `to_step_shell()`, `to_step_shell_fallback()`, `to_truck_shell_stitched()`, `to_truck_shell_bfr_mixed()`.

## Recent Commits (Last 5)

1. `3c29cc7` -- fix: resolve all test errors and warnings
2. `bddcc6b` -- Test fixes & warnings
3. `17d3741` -- refactor: use functional collect() patterns and add rayon parallelization
4. `0de9fb3` -- Handle boundary regular patches and clean example logging
5. `7b7783d` -- fix: update truck tests to work with new API changes

## Active Decisions

- **Superpatch merging** is the preferred strategy for creased models (reduces patch count).
- **BFR mixed-mode** combines regular B-spline patches with BFR-evaluated extraordinary patches.
- **wgpu pipeline** uses 17 bindings (positions + 5 derivative slots), future-proofing for derivatives.
- **Per-vertex component cap** in WGSL shader is 32 (AIDEV-QUESTION in shader).

## Untracked Files (Not Yet Committed)

- `.claude/` directory
- `CLAUDE-CONTINUE.md`, `CLAUDE.md`
- `opensubdiv-petite/shaders/` directory
- `opensubdiv-petite/src/osd/wgpu.rs`
- `opensubdiv-petite/tests/wgpu_stencil.rs`
- Various `.step` output files from test runs
