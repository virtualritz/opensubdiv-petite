#![cfg(feature = "wgpu")]

use opensubdiv_petite::{far, osd};
use wgpu::util::DeviceExt;

fn request_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::default();
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: true,
    }))?;

    let limits = wgpu::Limits {
        max_storage_buffers_per_shader_stage: 16,
        ..wgpu::Limits::downlevel_defaults()
    };

    let (device, queue) = pollster::block_on(adapter.request_device(
        &wgpu::DeviceDescriptor {
            label: Some("opensubdiv-petite test device"),
            required_features: wgpu::Features::empty(),
            required_limits: limits,
            memory_hints: wgpu::MemoryHints::Performance,
        },
        None,
    ))
    .ok()?;

    Some((device, queue))
}

/// Read back a GPU buffer to a `Vec<f32>`.
fn readback_buffer(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    src: &wgpu::Buffer,
    size_bytes: u64,
) -> Vec<f32> {
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("readback"),
        size: size_bytes,
        usage: wgpu::BufferUsages::MAP_READ | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    encoder.copy_buffer_to_buffer(src, 0, &readback, 0, size_bytes);
    queue.submit(std::iter::once(encoder.finish()));

    let slice = readback.slice(..);
    let (tx, rx) = std::sync::mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        tx.send(result).unwrap();
    });
    device.poll(wgpu::Maintain::Wait);
    rx.recv().unwrap().unwrap();
    let data: Vec<f32> = bytemuck::cast_slice(&slice.get_mapped_range()).to_vec();
    drop(readback);
    data
}

#[test]
fn wgpu_stencil_matches_cpu() -> Result<(), Box<dyn std::error::Error>> {
    let (device, queue) = match request_device() {
        Some(d) => d,
        None => return Ok(()), // Skip if no backend is available.
    };

    // Simple cube topology reused from cpu test.
    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];

    let positions = [
        -0.5_f32, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5,
        0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5,
    ];

    let descriptor = far::TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let options = far::TopologyRefinerOptions::default();

    let mut refiner = far::TopologyRefiner::new(descriptor, options)?;
    refiner.refine_uniform(far::topology_refiner::UniformRefinementOptions {
        refinement_level: 1,
        ..Default::default()
    });

    let stencil_table = far::StencilTable::new(
        &refiner,
        far::StencilTableOptions {
            generate_offsets: true,
            ..Default::default()
        },
    )?;

    let n_coarse_verts = refiner.level(0).unwrap().vertex_count();
    let n_refined_verts = stencil_table.len();

    // CPU baseline.
    let mut cpu_src = osd::CpuVertexBuffer::new(3, n_coarse_verts)?;
    let mut cpu_dst = osd::CpuVertexBuffer::new(3, n_refined_verts)?;
    cpu_src.update_data(&positions, 0, n_coarse_verts)?;
    let desc = osd::BufferDescriptor::new(0, 3, 3)?;
    osd::cpu_evaluator::evaluate_stencils(&cpu_src, desc, &mut cpu_dst, desc, &stencil_table)?;
    let cpu_data = cpu_dst.bind_cpu_buffer()?.to_vec();

    // GPU path.
    let gpu_table = osd::wgpu::StencilTableGpu::from_cpu(&device, &stencil_table)?;
    let src_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("opensubdiv-petite test src"),
        contents: bytemuck::cast_slice(&positions),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });
    let dst_size_bytes = (n_refined_verts * 3 * std::mem::size_of::<f32>()) as u64;
    let dst_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("opensubdiv-petite test dst"),
        size: dst_size_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let pipeline =
        osd::wgpu::StencilEvalPipeline::new(&device, osd::wgpu::WgslModuleConfig::default());

    osd::wgpu::evaluate_stencils(
        &device,
        &queue,
        &pipeline,
        &gpu_table,
        &src_buffer,
        &dst_buffer,
        desc,
        desc,
        0..n_refined_verts as u32,
    )?;

    let gpu_data = readback_buffer(&device, &queue, &dst_buffer, dst_size_bytes);

    assert_eq!(gpu_data.len(), cpu_data.len());
    for (cpu, gpu) in cpu_data.iter().zip(gpu_data.iter()) {
        assert!((cpu - gpu).abs() < 1e-5, "cpu {cpu} vs gpu {gpu}");
    }

    Ok(())
}

#[test]
fn wgpu_evaluate_stencils_convenience() -> Result<(), Box<dyn std::error::Error>> {
    let (device, queue) = match request_device() {
        Some(d) => d,
        None => return Ok(()),
    };

    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];
    let positions = [
        -0.5_f32, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5,
        0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5,
    ];

    let descriptor = far::TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let mut refiner =
        far::TopologyRefiner::new(descriptor, far::TopologyRefinerOptions::default())?;
    refiner.refine_uniform(far::topology_refiner::UniformRefinementOptions {
        refinement_level: 1,
        ..Default::default()
    });

    let stencil_table = far::StencilTable::new(
        &refiner,
        far::StencilTableOptions {
            generate_offsets: true,
            ..Default::default()
        },
    )?;

    let n_refined = stencil_table.len();
    let desc = osd::BufferDescriptor::new(0, 3, 3)?;

    let gpu_table = osd::wgpu::StencilTableGpu::from_cpu(&device, &stencil_table)?;
    let pipeline =
        osd::wgpu::StencilEvalPipeline::new(&device, osd::wgpu::WgslModuleConfig::default());

    let src_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("src"),
        contents: bytemuck::cast_slice(&positions),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });
    let dst_size_bytes = (n_refined * 3 * std::mem::size_of::<f32>()) as u64;
    let dst_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("dst"),
        size: dst_size_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    osd::wgpu::evaluate_stencils(
        &device,
        &queue,
        &pipeline,
        &gpu_table,
        &src_buffer,
        &dst_buffer,
        desc,
        desc,
        0..n_refined as u32,
    )?;

    let gpu_data = readback_buffer(&device, &queue, &dst_buffer, dst_size_bytes);

    let n_coarse = 8;
    let mut cpu_src = osd::CpuVertexBuffer::new(3, n_coarse)?;
    let mut cpu_dst = osd::CpuVertexBuffer::new(3, n_refined)?;
    cpu_src.update_data(&positions, 0, n_coarse)?;
    osd::cpu_evaluator::evaluate_stencils(&cpu_src, desc, &mut cpu_dst, desc, &stencil_table)?;
    let cpu_data = cpu_dst.bind_cpu_buffer()?.to_vec();

    assert_eq!(gpu_data.len(), cpu_data.len());
    for (cpu, gpu) in cpu_data.iter().zip(gpu_data.iter()) {
        assert!((cpu - gpu).abs() < 1e-5, "cpu {cpu} vs gpu {gpu}");
    }

    Ok(())
}

#[test]
fn wgpu_limit_stencil_positions_match_cpu() -> Result<(), Box<dyn std::error::Error>> {
    let (device, queue) = match request_device() {
        Some(d) => d,
        None => return Ok(()),
    };

    let vertices_per_face = [4, 4, 4, 4, 4, 4];
    let face_vertices = [
        0, 1, 3, 2, 2, 3, 5, 4, 4, 5, 7, 6, 6, 7, 1, 0, 1, 7, 5, 3, 6, 0, 2, 4,
    ];
    let positions = [
        -0.5_f32, -0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5, 0.5, 0.5, 0.5, 0.5, -0.5, 0.5, -0.5, 0.5,
        0.5, -0.5, -0.5, -0.5, -0.5, 0.5, -0.5, -0.5,
    ];

    let descriptor = far::TopologyDescriptor::new(8, &vertices_per_face, &face_vertices)?;
    let mut refiner =
        far::TopologyRefiner::new(descriptor, far::TopologyRefinerOptions::default())?;
    refiner.refine_adaptive(far::AdaptiveRefinementOptions::default(), &[]);

    let s = [0.25_f32, 0.5, 0.75];
    let t = [0.25_f32, 0.5, 0.75];
    let locations = [far::LocationArray {
        ptex_index: 0,
        s: &s,
        t: &t,
    }];

    let limit_table = far::LimitStencilTable::new(
        &refiner,
        &locations,
        None,
        None,
        far::LimitStencilTableOptions {
            generate_1st_derivatives: true,
            ..Default::default()
        },
    )?;

    let n_stencils = limit_table.len();
    let n_control = limit_table.control_vertex_count();

    // CPU evaluation: apply limit stencil weights manually to get positions.
    let cpu_positions: Vec<f32> = {
        let sizes = limit_table.sizes();
        let offsets = limit_table.offsets();
        let indices = limit_table.control_indices();
        let weights = limit_table.weights();

        let mut result = vec![0.0f32; n_stencils * 3];
        for i in 0..n_stencils {
            let offset = offsets[i].0 as usize;
            let size = sizes[i] as usize;
            for j in 0..size {
                let cv = indices[offset + j].0 as usize;
                let w = weights[offset + j];
                for c in 0..3 {
                    result[i * 3 + c] += positions[cv * 3 + c] * w;
                }
            }
        }
        result
    };

    // GPU path: use encode_with_derivatives (positions only, no derivative
    // outputs).
    let gpu_table = osd::wgpu::LimitStencilTableGpu::from_cpu(&device, &limit_table)?;
    let pipeline =
        osd::wgpu::StencilEvalPipeline::new(&device, osd::wgpu::WgslModuleConfig::default());

    // Source buffer must cover all control vertices.
    let src_size = n_control * 3;
    let mut src_data = vec![0.0f32; src_size];
    for i in 0..8.min(n_control) {
        for c in 0..3 {
            src_data[i * 3 + c] = positions[i * 3 + c];
        }
    }

    let src_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("src"),
        contents: bytemuck::cast_slice(&src_data),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
    });
    let dst_size_bytes = (n_stencils * 3 * std::mem::size_of::<f32>()) as u64;
    let dst_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("dst"),
        size: dst_size_bytes,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });

    let desc = osd::BufferDescriptor::new(0, 3, 3)?;
    osd::wgpu::evaluate_stencils_with_derivatives(
        &device,
        &queue,
        &pipeline,
        &gpu_table,
        &src_buffer,
        &dst_buffer,
        desc,
        desc,
        None,
        None,
        0..n_stencils as u32,
    )?;

    let gpu_data = readback_buffer(&device, &queue, &dst_buffer, dst_size_bytes);

    assert_eq!(gpu_data.len(), cpu_positions.len());
    for (cpu, gpu) in cpu_positions.iter().zip(gpu_data.iter()) {
        assert!((cpu - gpu).abs() < 1e-4, "cpu {cpu} vs gpu {gpu}");
    }

    Ok(())
}
