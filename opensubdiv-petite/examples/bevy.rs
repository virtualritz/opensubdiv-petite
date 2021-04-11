use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::pipeline::PrimitiveTopology;
use opensubdiv_petite::{far, tri_mesh_buffers};

// Uniformly refine up to 'max level' of 3.
static MAX_LEVEL: usize = 3;

fn main() {
    App::build()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup.system())
        .add_system(rotator_system.system())
        .run();
}

/// this component indicates what entities should rotate
struct Rotator;

/// rotates the parent, which will result in the child also rotating
fn rotator_system(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<Rotator>>,
) {
    for mut transform in query.iter_mut() {
        transform.rotation *= Quat::from_rotation_y(3.0 * time.delta_seconds());
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // chamfered_tetrahedron
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(subdivided_chamfered_tetrahedron()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.25, 0.0)
                * Transform::from_scale(Vec3::new(2.0, 2.0, 2.0)),
            ..Default::default()
        })
        .insert(Rotator);
    // light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..Default::default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..Default::default()
    });
}

fn subdivided_chamfered_tetrahedron() -> Mesh {
    // Topology for a chamfered tetrahedron.
    // cT – in Conway notation.
    let vertices = [
        0.57735025f32,
        0.57735025,
        0.57735025,
        0.57735025,
        -0.57735025,
        -0.57735025,
        -0.57735025,
        0.57735025,
        -0.57735025,
        -0.57735025,
        -0.57735025,
        0.57735025,
        -0.2566001,
        0.5132003,
        -0.5132003,
        0.5132003,
        -0.2566001,
        -0.5132003,
        0.5132003,
        0.5132003,
        0.2566001,
        -0.5132003,
        -0.2566001,
        0.5132003,
        -0.5132003,
        0.5132003,
        -0.2566001,
        0.2566001,
        0.5132003,
        0.5132003,
        0.5132003,
        -0.5132003,
        -0.2566001,
        -0.2566001,
        -0.5132003,
        0.5132003,
        0.5132003,
        0.2566001,
        0.5132003,
        -0.5132003,
        0.2566001,
        -0.5132003,
        -0.5132003,
        -0.5132003,
        0.2566001,
        0.2566001,
        -0.5132003,
        -0.5132003,
    ];

    assert!(0 == vertices.len() % 3);

    let face_arities = [3u32, 3, 3, 3, 6, 6, 6, 6, 6, 6];

    let face_vertices = [
        4u32, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 9, 8, 2, 4, 6, 0, 12,
        11, 3, 7, 9, 1, 15, 14, 3, 11, 10, 0, 6, 5, 1, 10, 12, 2, 8, 7, 3, 14,
        13, 1, 5, 4, 2, 13, 15,
    ];

    let creases = [
        4u32, 5, 5, 6, 7, 8, 8, 9, 10, 11, 11, 12, 13, 14, 14, 15, 0, 9, 2, 4,
        4, 6, 0, 12, 3, 7, 7, 9, 1, 15, 3, 11, 0, 6, 1, 10, 10, 12, 2, 8, 3,
        14, 1, 5, 2, 13, 13, 15,
    ];

    let crease_weights = [2.0; 24];

    // Create a refiner (a subdivider) from a topology descriptor.
    let mut refiner = far::TopologyRefiner::new(
        // Populate the descriptor with our raw data.
        far::TopologyDescriptor::new(
            vertices.len() / 3,
            &face_arities,
            &face_vertices,
        )
        .creases(&creases, &crease_weights)
        .left_handed(true)
        .clone(),
        far::TopologyRefinerOptions {
            scheme: far::Scheme::CatmullClark,
            boundary_interpolation: far::BoundaryInterpolation::EdgeOnly,
            ..Default::default()
        },
    )
    .expect("Could not create TopologyRefiner");

    // Refine (subdivice) the topology uniformy MAX_LEVEL
    // times.
    refiner.refine_uniform(far::topology_refiner::UniformRefinementOptions {
        refinement_level: MAX_LEVEL,
        ..Default::default()
    });

    // Interpolate vertex primvar data.
    let primvar_refiner = far::PrimvarRefiner::new(&refiner);

    let mut refined_vertices = vertices.to_vec();

    // Subdivide MAX_LEVEL times.
    // Note how the refined_vertices from the previous refinenemnet step become
    // the base for the next.
    for level in 1..=MAX_LEVEL {
        refined_vertices = primvar_refiner
            .interpolate(
                level,
                3, // Each element is a 3-tuple.
                &refined_vertices,
            )
            .unwrap();
    }

    // Convert the subdivison mesh (all quads by now) into disconnected
    // triangles.
    let (index, points, normals) = tri_mesh_buffers::to_triangle_mesh_buffers(
        &refined_vertices,
        refiner.level(MAX_LEVEL).unwrap().face_vertices_iter(),
    );

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
    // Bevy forces UVs. We create some fake UVs by just projecting through,
    // onto the XY plane.
    let uvs = points.iter().map(|p| [p[0], p[1]]).collect::<Vec<_>>();

    mesh.set_indices(Some(Indices::U32(index)));
    mesh.set_attribute(Mesh::ATTRIBUTE_POSITION, points);
    mesh.set_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.set_attribute(Mesh::ATTRIBUTE_UV_0, uvs);

    mesh
}
