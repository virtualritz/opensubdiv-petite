//! WGSL compute path for stencil evaluation.
//!
//! The WGSL source mirrors `glslComputeKernel.glsl` and is intended to be
//! renderer-agnostic (usable with Bevy/wgpu on native and wasm targets).
//! Pipeline setup and Bevy systems will live here; the shader source is the
//! canonical version.

use crate::far::{LimitStencilTable, StencilTable};
use crate::osd::BufferDescriptor;
use crate::{Error, Result};

use std::borrow::Cow;
use std::collections::HashMap;
use std::num::NonZeroU32;

use bytemuck::{bytes_of, Pod, Zeroable};
use thiserror::Error;
use wgpu::util::DeviceExt;

/// Canonical WGSL for stencil evaluation (positions + optional derivatives).
pub const STENCIL_EVAL_WGSL: &str = include_str!("../../shaders/wgsl/stencil_eval.wgsl");

/// Parameters used to configure the WGSL module creation.
#[derive(Debug, Clone)]
pub struct WgslModuleConfig {
    /// Workgroup size to bake into the specialization constant.
    pub workgroup_size: NonZeroU32,
}

impl Default for WgslModuleConfig {
    fn default() -> Self {
        Self {
            workgroup_size: NonZeroU32::new(64).expect("non-zero workgroup size"),
        }
    }
}

impl WgslModuleConfig {
    /// Create the `wgpu` shader module with the requested workgroup size baked
    /// via pipeline constants.
    pub fn create_shader_module(&self, device: &wgpu::Device) -> wgpu::ShaderModule {
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("opensubdiv-petite::stencil_eval_wgsl"),
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(STENCIL_EVAL_WGSL)),
        })
    }

    /// Return pipeline constants to override `WORKGROUP_SIZE`.
    pub fn pipeline_constants(&self) -> HashMap<String, f64> {
        let mut constants = HashMap::new();
        constants.insert("WORKGROUP_SIZE".into(), self.workgroup_size.get() as f64);
        constants
    }
}

/// Errors specific to the WGSL compute evaluator.
#[derive(Debug, Error)]
pub enum WgpuError {
    /// Primvar length exceeds shader static storage.
    #[error("Primvar length {length} exceeds WGSL kernel capacity ({max})")]
    PrimvarLengthExceeded { length: u32, max: u32 },

    /// Unsupported negative sizes/indices coming from the stencil table.
    #[error("Stencil table contains negative entries")]
    NegativeStencilEntry,

    /// Stencil table values exceed 32-bit limits.
    #[error("Stencil table entry {value} exceeds u32 capacity")]
    StencilEntryTooLarge { value: usize },

    /// Stencil offsets are missing but required for compute evaluation.
    #[error("Stencil table is missing offsets; enable generate_offsets in StencilTableOptions")]
    MissingOffsets,
}

/// Result alias for WGSL compute operations.
pub type WgpuResult<T> = std::result::Result<T, WgpuError>;

/// GPU-side stencil table buffers (storage buffer views).
#[derive(Debug)]
pub struct StencilTableGpu {
    pub stencil_count: u32,
    pub sizes: wgpu::Buffer,
    pub offsets: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub weights: wgpu::Buffer,
}

impl StencilTableGpu {
    /// Upload the host `StencilTable` data into GPU storage buffers.
    pub fn from_cpu(device: &wgpu::Device, table: &StencilTable) -> WgpuResult<Self> {
        let sizes = table.sizes();
        let offsets = table.offsets();
        let indices = table.control_indices();
        let weights = table.weights();

        if sizes.iter().any(|s| *s < 0) {
            return Err(WgpuError::NegativeStencilEntry);
        }

        if offsets.is_empty() {
            return Err(WgpuError::MissingOffsets);
        }

        let sizes_u32: Vec<u32> = sizes.iter().map(|v| *v as u32).collect();
        // AIDEV-NOTE: offsets and indices are Index(u32), already non-negative.
        let offsets_u32: Vec<u32> = offsets.iter().map(|v| v.0).collect();
        let indices_u32: Vec<u32> = indices.iter().map(|v| v.0).collect();

        let sizes_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("opensubdiv-petite::stencil_sizes"),
            contents: bytemuck::cast_slice(&sizes_u32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let offsets_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("opensubdiv-petite::stencil_offsets"),
            contents: bytemuck::cast_slice(&offsets_u32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let indices_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("opensubdiv-petite::stencil_indices"),
            contents: bytemuck::cast_slice(&indices_u32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let weights_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("opensubdiv-petite::stencil_weights"),
            contents: bytemuck::cast_slice(weights),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        Ok(Self {
            stencil_count: table.len() as u32,
            sizes: sizes_buf,
            offsets: offsets_buf,
            indices: indices_buf,
            weights: weights_buf,
        })
    }
}

/// Upload a float slice as a storage buffer, or a 4-byte zero buffer if empty.
fn upload_weight_buffer(device: &wgpu::Device, label: &str, data: &[f32]) -> wgpu::Buffer {
    if data.is_empty() {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &[0u8; 4],
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    } else {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::cast_slice(data),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }
}

/// GPU-side limit stencil table buffers (base + 5 derivative weight buffers).
#[derive(Debug)]
pub struct LimitStencilTableGpu {
    /// Base stencil table buffers (sizes, offsets, indices, weights).
    pub base: StencilTableGpu,
    /// du derivative weights.
    pub du_weights: wgpu::Buffer,
    /// dv derivative weights.
    pub dv_weights: wgpu::Buffer,
    /// duu derivative weights.
    pub duu_weights: wgpu::Buffer,
    /// duv derivative weights.
    pub duv_weights: wgpu::Buffer,
    /// dvv derivative weights.
    pub dvv_weights: wgpu::Buffer,
}

impl LimitStencilTableGpu {
    /// Upload a [`LimitStencilTable`] into GPU storage buffers.
    pub fn from_cpu(device: &wgpu::Device, table: &LimitStencilTable) -> WgpuResult<Self> {
        let sizes = table.sizes();
        let offsets = table.offsets();
        let indices = table.control_indices();
        let weights = table.weights();

        if sizes.iter().any(|s| *s < 0) {
            return Err(WgpuError::NegativeStencilEntry);
        }

        if offsets.is_empty() {
            return Err(WgpuError::MissingOffsets);
        }

        let sizes_u32: Vec<u32> = sizes.iter().map(|v| *v as u32).collect();
        let offsets_u32: Vec<u32> = offsets.iter().map(|v| v.0).collect();
        let indices_u32: Vec<u32> = indices.iter().map(|v| v.0).collect();

        let sizes_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("opensubdiv-petite::limit_stencil_sizes"),
            contents: bytemuck::cast_slice(&sizes_u32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let offsets_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("opensubdiv-petite::limit_stencil_offsets"),
            contents: bytemuck::cast_slice(&offsets_u32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let indices_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("opensubdiv-petite::limit_stencil_indices"),
            contents: bytemuck::cast_slice(&indices_u32),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });
        let weights_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("opensubdiv-petite::limit_stencil_weights"),
            contents: bytemuck::cast_slice(weights),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        });

        let base = StencilTableGpu {
            stencil_count: table.len() as u32,
            sizes: sizes_buf,
            offsets: offsets_buf,
            indices: indices_buf,
            weights: weights_buf,
        };

        let du_weights =
            upload_weight_buffer(device, "opensubdiv-petite::du_weights", table.du_weights());
        let dv_weights =
            upload_weight_buffer(device, "opensubdiv-petite::dv_weights", table.dv_weights());
        let duu_weights = upload_weight_buffer(
            device,
            "opensubdiv-petite::duu_weights",
            table.duu_weights(),
        );
        let duv_weights = upload_weight_buffer(
            device,
            "opensubdiv-petite::duv_weights",
            table.duv_weights(),
        );
        let dvv_weights = upload_weight_buffer(
            device,
            "opensubdiv-petite::dvv_weights",
            table.dvv_weights(),
        );

        Ok(Self {
            base,
            du_weights,
            dv_weights,
            duu_weights,
            duv_weights,
            dvv_weights,
        })
    }
}

/// Derivative output buffers provided by the caller.
pub struct DerivativeOutputBuffers<'a> {
    /// Output buffer for du derivatives.
    pub du: &'a wgpu::Buffer,
    /// Output buffer for dv derivatives.
    pub dv: &'a wgpu::Buffer,
    /// Output buffer for duu derivatives (optional).
    pub duu: Option<&'a wgpu::Buffer>,
    /// Output buffer for duv derivatives (optional).
    pub duv: Option<&'a wgpu::Buffer>,
    /// Output buffer for dvv derivatives (optional).
    pub dvv: Option<&'a wgpu::Buffer>,
}

/// [`BufferDescriptor`] layout for each derivative output.
#[derive(Debug, Clone, Copy)]
pub struct DerivativeDescriptors {
    /// Descriptor for du output.
    pub du: BufferDescriptor,
    /// Descriptor for dv output.
    pub dv: BufferDescriptor,
    /// Descriptor for duu output (optional).
    pub duu: Option<BufferDescriptor>,
    /// Descriptor for duv output (optional).
    pub duv: Option<BufferDescriptor>,
    /// Descriptor for dvv output (optional).
    pub dvv: Option<BufferDescriptor>,
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ShaderParams {
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

impl ShaderParams {
    fn from_descriptors(
        src_desc: BufferDescriptor,
        dst_desc: BufferDescriptor,
        batch_start: u32,
        batch_end: u32,
    ) -> WgpuResult<Self> {
        let length = dst_desc.0.length as u32;
        if length > 32 {
            return Err(WgpuError::PrimvarLengthExceeded { length, max: 32 });
        }

        Ok(Self {
            src_offset: src_desc.0.offset as u32,
            dst_offset: dst_desc.0.offset as u32,
            src_stride: src_desc.0.stride as u32,
            dst_stride: dst_desc.0.stride as u32,
            length,
            batch_start,
            batch_end,
            du_offset: 0,
            du_stride: 0,
            du_length: 0,
            dv_offset: 0,
            dv_stride: 0,
            dv_length: 0,
            duu_offset: 0,
            duu_stride: 0,
            duu_length: 0,
            duv_offset: 0,
            duv_stride: 0,
            duv_length: 0,
            dvv_offset: 0,
            dvv_stride: 0,
            dvv_length: 0,
        })
    }

    fn from_descriptors_with_derivatives(
        src_desc: BufferDescriptor,
        dst_desc: BufferDescriptor,
        deriv_descs: Option<&DerivativeDescriptors>,
        batch_start: u32,
        batch_end: u32,
    ) -> WgpuResult<Self> {
        let mut params = Self::from_descriptors(src_desc, dst_desc, batch_start, batch_end)?;

        if let Some(dd) = deriv_descs {
            params.du_offset = dd.du.0.offset as u32;
            params.du_stride = dd.du.0.stride as u32;
            params.du_length = dd.du.0.length as u32;
            params.dv_offset = dd.dv.0.offset as u32;
            params.dv_stride = dd.dv.0.stride as u32;
            params.dv_length = dd.dv.0.length as u32;

            if let Some(duu) = dd.duu {
                params.duu_offset = duu.0.offset as u32;
                params.duu_stride = duu.0.stride as u32;
                params.duu_length = duu.0.length as u32;
            }
            if let Some(duv) = dd.duv {
                params.duv_offset = duv.0.offset as u32;
                params.duv_stride = duv.0.stride as u32;
                params.duv_length = duv.0.length as u32;
            }
            if let Some(dvv) = dd.dvv {
                params.dvv_offset = dvv.0.offset as u32;
                params.dvv_stride = dvv.0.stride as u32;
                params.dvv_length = dvv.0.length as u32;
            }
        }

        Ok(params)
    }
}

/// Compute pipeline + layouts for stencil evaluation.
#[derive(Debug)]
pub struct StencilEvalPipeline {
    _shader: wgpu::ShaderModule,
    bind_group_layout: wgpu::BindGroupLayout,
    pipeline: wgpu::ComputePipeline,
    workgroup_size: NonZeroU32,
}

impl StencilEvalPipeline {
    pub fn new(device: &wgpu::Device, config: WgslModuleConfig) -> Self {
        let shader = config.create_shader_module(device);

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("opensubdiv-petite::stencil_eval_bgl"),
            entries: &[
                // 0: uniform params
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: std::num::NonZeroU64::new(
                            std::mem::size_of::<ShaderParams>() as u64,
                        ),
                    },
                    count: None,
                },
                // 1: src (read-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 2: dst (read-write)
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 3: sizes
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 4: offsets
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 5: indices
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 6: weights
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 7-11: derivative weights (read-only)
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 9,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 10,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 11,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                // 12-16: derivative outputs (read-write)
                wgpu::BindGroupLayoutEntry {
                    binding: 12,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 13,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 14,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 15,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 16,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: false },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("opensubdiv-petite::stencil_eval_pipeline_layout"),
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let constants = config.pipeline_constants();
        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("opensubdiv-petite::stencil_eval_pipeline"),
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: Some("eval_stencils"),
            compilation_options: wgpu::PipelineCompilationOptions {
                constants: &constants,
                zero_initialize_workgroup_memory: true,
            },
            cache: None,
        });

        Self {
            _shader: shader,
            bind_group_layout,
            pipeline,
            workgroup_size: config.workgroup_size,
        }
    }

    fn empty_buffer(device: &wgpu::Device, label: &str) -> wgpu::Buffer {
        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: &[0u8; 4],
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        })
    }

    /// Encode a stencil evaluation dispatch.
    #[allow(clippy::too_many_arguments)]
    pub fn encode(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        gpu_table: &StencilTableGpu,
        src_buffer: &wgpu::Buffer,
        dst_buffer: &wgpu::Buffer,
        src_desc: BufferDescriptor,
        dst_desc: BufferDescriptor,
        batch_range: std::ops::Range<u32>,
    ) -> Result<()> {
        let params =
            ShaderParams::from_descriptors(src_desc, dst_desc, batch_range.start, batch_range.end)
                .map_err(|e| Error::Ffi(e.to_string()))?;
        let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("opensubdiv-petite::stencil_params"),
            contents: bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let zero_weights = Self::empty_buffer(device, "opensubdiv-petite::zero_weights");
        let zero_output = Self::empty_buffer(device, "opensubdiv-petite::zero_derivative");

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("opensubdiv-petite::stencil_eval_bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: src_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: dst_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: gpu_table.sizes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: gpu_table.offsets.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: gpu_table.indices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: gpu_table.weights.as_entire_binding(),
                },
                // Derivative weights: not yet wired; bind zero buffers.
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: zero_weights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: zero_weights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: zero_weights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: zero_weights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: zero_weights.as_entire_binding(),
                },
                // Derivative outputs: not yet wired; bind dummy outputs.
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: zero_output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 13,
                    resource: zero_output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 14,
                    resource: zero_output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 15,
                    resource: zero_output.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 16,
                    resource: zero_output.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("opensubdiv-petite::stencil_eval"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);

        let invocations = batch_range.end - batch_range.start;
        let groups = invocations.div_ceil(self.workgroup_size.get());
        pass.dispatch_workgroups(groups, 1, 1);
        drop(pass);

        Ok(())
    }

    /// Encode a stencil evaluation dispatch with derivative outputs.
    // AIDEV-NOTE: Derivative binding scheme:
    // Bindings 7-11: derivative weight buffers (du, dv, duu, duv, dvv) from
    // LimitStencilTableGpu. Bindings 12-16: derivative output buffers provided
    // by the caller (or zero buffers if None). The shader gates on
    // `deriv_length > 0` so zero-length descriptors effectively disable
    // derivative computation.
    #[allow(clippy::too_many_arguments)]
    pub fn encode_with_derivatives(
        &self,
        device: &wgpu::Device,
        encoder: &mut wgpu::CommandEncoder,
        gpu_table: &LimitStencilTableGpu,
        src_buffer: &wgpu::Buffer,
        dst_buffer: &wgpu::Buffer,
        src_desc: BufferDescriptor,
        dst_desc: BufferDescriptor,
        deriv_outputs: Option<&DerivativeOutputBuffers<'_>>,
        deriv_descs: Option<&DerivativeDescriptors>,
        batch_range: std::ops::Range<u32>,
    ) -> Result<()> {
        let params = ShaderParams::from_descriptors_with_derivatives(
            src_desc,
            dst_desc,
            deriv_descs,
            batch_range.start,
            batch_range.end,
        )
        .map_err(|e| Error::Ffi(e.to_string()))?;

        let params_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("opensubdiv-petite::stencil_params"),
            contents: bytes_of(&params),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let zero_buf = Self::empty_buffer(device, "opensubdiv-petite::zero_derivative");

        let du_out = deriv_outputs.map(|d| d.du).unwrap_or(&zero_buf);
        let dv_out = deriv_outputs.map(|d| d.dv).unwrap_or(&zero_buf);
        let duu_out = deriv_outputs.and_then(|d| d.duu).unwrap_or(&zero_buf);
        let duv_out = deriv_outputs.and_then(|d| d.duv).unwrap_or(&zero_buf);
        let dvv_out = deriv_outputs.and_then(|d| d.dvv).unwrap_or(&zero_buf);

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("opensubdiv-petite::stencil_eval_deriv_bg"),
            layout: &self.bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: params_buf.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: src_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: dst_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: gpu_table.base.sizes.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: gpu_table.base.offsets.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: gpu_table.base.indices.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: gpu_table.base.weights.as_entire_binding(),
                },
                // Derivative weight buffers.
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: gpu_table.du_weights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: gpu_table.dv_weights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: gpu_table.duu_weights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 10,
                    resource: gpu_table.duv_weights.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: gpu_table.dvv_weights.as_entire_binding(),
                },
                // Derivative output buffers.
                wgpu::BindGroupEntry {
                    binding: 12,
                    resource: du_out.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 13,
                    resource: dv_out.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 14,
                    resource: duu_out.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 15,
                    resource: duv_out.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 16,
                    resource: dvv_out.as_entire_binding(),
                },
            ],
        });

        let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: Some("opensubdiv-petite::stencil_eval_derivs"),
            timestamp_writes: None,
        });
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &bind_group, &[]);

        let invocations = batch_range.end - batch_range.start;
        let groups = invocations.div_ceil(self.workgroup_size.get());
        pass.dispatch_workgroups(groups, 1, 1);
        drop(pass);

        Ok(())
    }
}

/// One-shot convenience: encode, submit, and wait for stencil evaluation.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_stencils(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &StencilEvalPipeline,
    gpu_table: &StencilTableGpu,
    src_buffer: &wgpu::Buffer,
    dst_buffer: &wgpu::Buffer,
    src_desc: BufferDescriptor,
    dst_desc: BufferDescriptor,
    batch_range: std::ops::Range<u32>,
) -> Result<()> {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("opensubdiv-petite::evaluate_stencils"),
    });
    pipeline.encode(
        device,
        &mut encoder,
        gpu_table,
        src_buffer,
        dst_buffer,
        src_desc,
        dst_desc,
        batch_range,
    )?;
    queue.submit(std::iter::once(encoder.finish()));
    device.poll(wgpu::Maintain::Wait);
    Ok(())
}

/// One-shot convenience: encode, submit, and wait for stencil evaluation with
/// derivative outputs.
#[allow(clippy::too_many_arguments)]
pub fn evaluate_stencils_with_derivatives(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &StencilEvalPipeline,
    gpu_table: &LimitStencilTableGpu,
    src_buffer: &wgpu::Buffer,
    dst_buffer: &wgpu::Buffer,
    src_desc: BufferDescriptor,
    dst_desc: BufferDescriptor,
    deriv_outputs: Option<&DerivativeOutputBuffers<'_>>,
    deriv_descs: Option<&DerivativeDescriptors>,
    batch_range: std::ops::Range<u32>,
) -> Result<()> {
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("opensubdiv-petite::evaluate_stencils_with_derivs"),
    });
    pipeline.encode_with_derivatives(
        device,
        &mut encoder,
        gpu_table,
        src_buffer,
        dst_buffer,
        src_desc,
        dst_desc,
        deriv_outputs,
        deriv_descs,
        batch_range,
    )?;
    queue.submit(std::iter::once(encoder.finish()));
    device.poll(wgpu::Maintain::Wait);
    Ok(())
}
