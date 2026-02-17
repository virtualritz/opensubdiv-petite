// Stencil evaluation compute kernel translated from glslComputeKernel.glsl.
// This is the canonical WGSL source; host code sets specialization constants
// and bindings to match the OpenSubdiv stencil table layout.

override WORKGROUP_SIZE: u32 = 64u;

struct Params {
    src_offset: u32,
    dst_offset: u32,
    src_stride: u32,
    dst_stride: u32,
    length: u32,
    batch_start: u32,
    batch_end: u32,
    du_offset: u32,
    du_stride: u32,
    du_length: u32,
    dv_offset: u32,
    dv_stride: u32,
    dv_length: u32,
    duu_offset: u32,
    duu_stride: u32,
    duu_length: u32,
    duv_offset: u32,
    duv_stride: u32,
    duv_length: u32,
    dvv_offset: u32,
    dvv_stride: u32,
    dvv_length: u32,
}

@group(0) @binding(0)
var<uniform> params: Params;

@group(0) @binding(1)
var<storage, read> src_buffer: array<f32>;

@group(0) @binding(2)
var<storage, read_write> dst_buffer: array<f32>;

@group(0) @binding(3)
var<storage, read> stencil_sizes: array<u32>;

@group(0) @binding(4)
var<storage, read> stencil_offsets: array<u32>;

@group(0) @binding(5)
var<storage, read> stencil_indices: array<u32>;

@group(0) @binding(6)
var<storage, read> stencil_weights: array<f32>;

// Derivative weights (optional; length 0 disables writes).
@group(0) @binding(7)
var<storage, read> du_weights: array<f32>;

@group(0) @binding(8)
var<storage, read> dv_weights: array<f32>;

@group(0) @binding(9)
var<storage, read> duu_weights: array<f32>;

@group(0) @binding(10)
var<storage, read> duv_weights: array<f32>;

@group(0) @binding(11)
var<storage, read> dvv_weights: array<f32>;

// Derivative outputs (optional; length 0 disables writes).
@group(0) @binding(12)
var<storage, read_write> du_buffer: array<f32>;

@group(0) @binding(13)
var<storage, read_write> dv_buffer: array<f32>;

@group(0) @binding(14)
var<storage, read_write> duu_buffer: array<f32>;

@group(0) @binding(15)
var<storage, read_write> duv_buffer: array<f32>;

@group(0) @binding(16)
var<storage, read_write> dvv_buffer: array<f32>;

// AIDEV-NOTE: We cap per-vertex component storage at 32.
// Increase if primvar arity exceeds 32 components.
const MAX_LENGTH: u32 = 32u;

@compute @workgroup_size(WORKGROUP_SIZE)
fn eval_stencils(@builtin(global_invocation_id) gid: vec3<u32>) {
    let current = gid.x + params.batch_start;
    if (current >= params.batch_end) {
        return;
    }

    let stencil_offset = stencil_offsets[current];
    let stencil_count = stencil_sizes[current];
    let dst_base = params.dst_offset + current * params.dst_stride;

    // Position.
    for (var c: u32 = 0u; c < params.length && c < MAX_LENGTH; c = c + 1u) {
        var sum: f32 = 0.0;
        for (var i: u32 = 0u; i < stencil_count; i = i + 1u) {
            let si = stencil_offset + i;
            let vi = params.src_offset + stencil_indices[si] * params.src_stride + c;
            sum = sum + stencil_weights[si] * src_buffer[vi];
        }
        dst_buffer[dst_base + c] = sum;
    }

    // First derivatives.
    if (params.du_length > 0u) {
        let du_base = params.du_offset + current * params.du_stride;
        for (var c: u32 = 0u; c < params.du_length && c < MAX_LENGTH; c = c + 1u) {
            var sum: f32 = 0.0;
            for (var i: u32 = 0u; i < stencil_count; i = i + 1u) {
                let si = stencil_offset + i;
                let vi = params.src_offset + stencil_indices[si] * params.src_stride + c;
                sum = sum + du_weights[si] * src_buffer[vi];
            }
            du_buffer[du_base + c] = sum;
        }
    }

    if (params.dv_length > 0u) {
        let dv_base = params.dv_offset + current * params.dv_stride;
        for (var c: u32 = 0u; c < params.dv_length && c < MAX_LENGTH; c = c + 1u) {
            var sum: f32 = 0.0;
            for (var i: u32 = 0u; i < stencil_count; i = i + 1u) {
                let si = stencil_offset + i;
                let vi = params.src_offset + stencil_indices[si] * params.src_stride + c;
                sum = sum + dv_weights[si] * src_buffer[vi];
            }
            dv_buffer[dv_base + c] = sum;
        }
    }

    // Second derivatives.
    if (params.duu_length > 0u) {
        let duu_base = params.duu_offset + current * params.duu_stride;
        for (var c: u32 = 0u; c < params.duu_length && c < MAX_LENGTH; c = c + 1u) {
            var sum: f32 = 0.0;
            for (var i: u32 = 0u; i < stencil_count; i = i + 1u) {
                let si = stencil_offset + i;
                let vi = params.src_offset + stencil_indices[si] * params.src_stride + c;
                sum = sum + duu_weights[si] * src_buffer[vi];
            }
            duu_buffer[duu_base + c] = sum;
        }
    }

    if (params.duv_length > 0u) {
        let duv_base = params.duv_offset + current * params.duv_stride;
        for (var c: u32 = 0u; c < params.duv_length && c < MAX_LENGTH; c = c + 1u) {
            var sum: f32 = 0.0;
            for (var i: u32 = 0u; i < stencil_count; i = i + 1u) {
                let si = stencil_offset + i;
                let vi = params.src_offset + stencil_indices[si] * params.src_stride + c;
                sum = sum + duv_weights[si] * src_buffer[vi];
            }
            duv_buffer[duv_base + c] = sum;
        }
    }

    if (params.dvv_length > 0u) {
        let dvv_base = params.dvv_offset + current * params.dvv_stride;
        for (var c: u32 = 0u; c < params.dvv_length && c < MAX_LENGTH; c = c + 1u) {
            var sum: f32 = 0.0;
            for (var i: u32 = 0u; i < stencil_count; i = i + 1u) {
                let si = stencil_offset + i;
                let vi = params.src_offset + stencil_indices[si] * params.src_stride + c;
                sum = sum + dvv_weights[si] * src_buffer[vi];
            }
            dvv_buffer[dvv_base + c] = sum;
        }
    }
}
