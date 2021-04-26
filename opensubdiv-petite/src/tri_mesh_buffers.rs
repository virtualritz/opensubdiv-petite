//! # Triangle Buffer Conversion
//!
//! Helper for turning *OpenSubdiv* geometry into triangle mesh buffers for use
//! with realtime rendering.
use itertools::Itertools;
use slice_of_array::prelude::*;

static EPSILON: f32 = 0.00000001;

type Vector = ultraviolet::vec::Vec3;
type Normal = Vector;
type Point = Vector;

use crate::far::FaceVerticesIterator;

/// Returns a flat [`u32`] triangle index buffer and two, flat matching point
/// and normal buffers.
///
/// All the faces are disconnected. I.e. points & normals are duplicated for
/// each shared vertex.
pub fn to_triangle_mesh_buffers<'a>(
    vertices: &[f32],
    face_vertices: impl Into<FaceVerticesIterator<'a>> + Iterator,
) -> (Vec<u32>, Vec<[f32; 3]>, Vec<[f32; 3]>) {
    let face_vertices_iter = face_vertices.into();

    #[cfg(feature = "topology_validation")]
    for face in face_vertices_iter.clone() {
        for index in face {
            if vertices.len() <= (3 * index + 2) as usize {
                panic!("Vertex index {} is out of bounds.", index);
            }
        }
    }

    let points_nested = vertices.nest::<[_; 3]>();

    let (points_nested, normals_nested): (Vec<[f32; 3]>, Vec<[f32; 3]>) =
        face_vertices_iter
            .clone()
            .flat_map(|face| {
                face.iter()
                    // Grab the three vertex index entries.
                    .circular_tuple_windows::<(_, _, _)>()
                    .map(|(&i0, &i1, &i2)| {
                        let i0 = i0 as usize;
                        let i1 = i1 as usize;
                        let i2 = i2 as usize;
                        // The middle point of our tuple
                        let point = Point::new(
                            points_nested[i1][0],
                            points_nested[i1][1],
                            points_nested[i1][2],
                        );
                        // Create a normal from that
                        let normal = -orthogonal(
                            &Point::new(
                                points_nested[i0][0],
                                points_nested[i0][1],
                                points_nested[i0][2],
                            ),
                            &point,
                            &Point::new(
                                points_nested[i2][0],
                                points_nested[i2][1],
                                points_nested[i2][2],
                            ),
                        );
                        let mag_sq = normal.mag_sq();

                        // Check for collinearity:
                        let normal = if mag_sq < EPSILON as _ {
                            -face_normal(&index_as_points(face, points_nested))
                                .unwrap()
                        } else {
                            -normal / mag_sq.sqrt()
                        };

                        (
                            [point.x, point.y, point.z],
                            [normal.x, normal.y, normal.z],
                        )
                    })
                    .collect_vec()
            })
            .unzip();

    // Build a new face index. Same topology as the old one, only with new keys.
    let triangle_face_index = face_vertices_iter
        // Build a new index where each face has the original arity and the new
        // numbering.
        .scan(0.., |counter, face| {
            Some(counter.take(face.len()).collect::<Vec<u32>>())
        })
        // Now split each of these faces into triangles.
        .flat_map(|face| {
            debug_assert!(face.len() < 5);
            // Bitriangulate quadrilateral faces use shortest diagonal so
            // triangles are most nearly equilateral.
            if 4 == face.len() {
                let p = index_as_points(&face, points_nested.as_slice());

                if (p[0] - p[2]).mag_sq() < (p[1] - p[3]).mag_sq() {
                    vec![face[0], face[1], face[2], face[0], face[2], face[3]]
                } else {
                    vec![face[1], face[2], face[3], face[1], face[3], face[0]]
                }
            } else {
                // It's a triangle -> just forward.
                face.to_vec()
            }
        })
        .collect();

    (
        triangle_face_index,
        points_nested.to_vec(),
        normals_nested.to_vec(),
    )
}

#[inline]
fn orthogonal(v0: &Point, v1: &Point, v2: &Point) -> Vector {
    (*v1 - *v0).cross(*v2 - *v1)
}

#[inline]
pub(crate) fn index_as_points(face: &[u32], points: &[[f32; 3]]) -> Vec<Point> {
    face.iter()
        .map(|index| {
            let index = *index as usize;
            Point::new(points[index][0], points[index][1], points[index][2])
        })
        .collect()
}

/// Computes the normal of a face.
/// Tries to do the right thing if the face
/// is non-planar or degenerate.
#[inline]
fn face_normal(points: &[Point]) -> Option<Normal> {
    let mut considered_edges = 0;

    let normal = points.iter().circular_tuple_windows::<(_, _, _)>().fold(
        Vector::zero(),
        |normal, corner| {
            considered_edges += 1;
            let ortho_normal = orthogonal(&corner.0, &corner.1, &corner.2);
            let mag_sq = ortho_normal.mag_sq();
            // Filter out collinear edge pairs.
            if mag_sq < EPSILON as _ {
                normal
            } else {
                // Subtract normalized ortho_normal.
                normal - ortho_normal / mag_sq.sqrt()
            }
        },
    );

    if 0 == considered_edges {
        // Degenerate/zero size face.
        None
    } else {
        Some(normal / considered_edges as f32)
    }
}
