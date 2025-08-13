use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use opensubdiv_petite::{far, tri_mesh_buffers};
use smooth_bevy_cameras::{
    controllers::orbit::{OrbitCameraBundle, OrbitCameraController, OrbitCameraPlugin},
    LookTransformPlugin,
};

// Uniformly refine up to 'max level' of 3.
static MAX_LEVEL: usize = 3;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(LookTransformPlugin)
        .add_plugins(OrbitCameraPlugin::default())
        .add_systems(Startup, setup)
        .add_systems(Update, (rotator_system, close_on_esc))
        .run();
}

/// this component indicates what entities should rotate
#[derive(Component)]
struct Rotator;

/// rotates the parent, which will result in the child also rotating
fn rotator_system(time: Res<Time>, mut query: Query<&mut Transform, With<Rotator>>) {
    for mut transform in &mut query {
        transform.rotate_x(3.0 * time.delta_secs());
    }
}

fn close_on_esc(mut exit: EventWriter<AppExit>, keyboard: Res<ButtonInput<KeyCode>>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        exit.write(AppExit::Success);
    }
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut ambient_light: ResMut<AmbientLight>,
) {
    // Add ambient light so materials are visible
    ambient_light.brightness = 300.0;
    ambient_light.color = Color::WHITE;
    // chamfered_tetrahedron

    commands.spawn((
        Mesh3d(meshes.add(subdivided_chamfered_tetrahedron())),
        MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
        Transform::from_xyz(0.0, 0.25, 0.0).with_scale(Vec3::splat(2.0)),
        Rotator,
    ));

    // Add a test cube to verify materials work
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.0, 1.0, 1.0))),
        MeshMaterial3d(materials.add(Color::srgb(0.3, 0.3, 0.9))),
        Transform::from_xyz(3.0, 0.5, 0.0),
    ));

    // light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // camera
    commands
        .spawn((
            Camera3d::default(),
            Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
        ))
        .insert(OrbitCameraBundle::new(
            OrbitCameraController::default(),
            Vec3::new(-2.0, 2.5, 5.0),
            Vec3::new(0., 0., 0.),
            Vec3::new(0., -1., 0.),
        ));
}

fn subdivided_chamfered_tetrahedron() -> Mesh {
    eprintln!("Starting subdivided_chamfered_tetrahedron");
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
        4u32, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 0, 9, 8, 2, 4, 6, 0, 12, 11, 3, 7, 9, 1, 15,
        14, 3, 11, 10, 0, 6, 5, 1, 10, 12, 2, 8, 7, 3, 14, 13, 1, 5, 4, 2, 13, 15,
    ];

    let creases = [
        4u32, 5, 5, 6, 7, 8, 8, 9, 10, 11, 11, 12, 13, 14, 14, 15, 0, 9, 2, 4, 4, 6, 0, 12, 3, 7,
        7, 9, 1, 15, 3, 11, 0, 6, 1, 10, 10, 12, 2, 8, 3, 14, 1, 5, 2, 13, 13, 15,
    ];

    let crease_weights = [2.0; 24];

    // Create a refiner (a subdivider) from a topology descriptor.
    eprintln!("Creating TopologyDescriptor");
    let mut descriptor = far::TopologyDescriptor::new(vertices.len() / 3, &face_arities, &face_vertices)
        .expect("Could not create TopologyDescriptor");
    descriptor.creases(&creases, &crease_weights);
    descriptor.left_handed(true);
    
    eprintln!("Creating TopologyRefiner");
    let mut refiner = far::TopologyRefiner::new(
        descriptor,
        far::TopologyRefinerOptions {
            scheme: far::Scheme::CatmullClark,
            boundary_interpolation: Some(far::BoundaryInterpolation::EdgeOnly),
            ..Default::default()
        },
    )
    .expect("Could not create TopologyRefiner");

    // Refine (subdivice) the topology uniformy MAX_LEVEL times.
    refiner.refine_uniform(far::topology_refiner::UniformRefinementOptions {
        refinement_level: MAX_LEVEL,
        ..Default::default()
    });

    // Interpolate vertex primvar data.
    eprintln!("Creating PrimvarRefiner");
    let primvar_refiner = far::PrimvarRefiner::new(&refiner).expect("Could not create PrimvarRefiner");

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

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::default());

    mesh.insert_indices(Indices::U32(index));
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, points);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    eprintln!("About to return from subdivided_chamfered_tetrahedron");
    mesh
}
