use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology, VertexAttributeValues};
use bevy::render::render_asset::RenderAssetUsages;

use crate::map::shapefile_loader::PolygonGroupData;

/// Build a Bevy [`Mesh`] (triangle list) for one [`PolygonGroupData`] by
/// ear-clipping the outer ring + optional holes via `earcutr`.
///
/// Returns `None` if the geometry is degenerate (< 3 valid vertices).
pub fn tessellate_group(group: &PolygonGroupData) -> Option<Mesh> {
    // -----------------------------------------------------------------------
    // 1. Flatten outer ring + holes into a single f64 vertex array.
    //    earcutr expects [x0, y0, x1, y1, ...] (dims = 2).
    // -----------------------------------------------------------------------
    let mut flat: Vec<f64> = Vec::with_capacity(
        (group.outer.len() + group.holes.iter().map(|h| h.len()).sum::<usize>()) * 2,
    );
    let mut hole_indices: Vec<usize> = Vec::with_capacity(group.holes.len());

    for (x, y) in &group.outer {
        flat.push(*x as f64);
        flat.push(*y as f64);
    }

    for hole in &group.holes {
        // Record the vertex index (not flat index) where this hole starts.
        hole_indices.push(flat.len() / 2);
        for (x, y) in hole {
            flat.push(*x as f64);
            flat.push(*y as f64);
        }
    }

    let vertex_count = flat.len() / 2;
    if vertex_count < 3 {
        return None;
    }

    // -----------------------------------------------------------------------
    // 2. Ear-clip triangulation.
    // -----------------------------------------------------------------------
    let tri_indices = earcutr::earcut(&flat, &hole_indices, 2).ok()?;
    if tri_indices.is_empty() {
        return None;
    }

    // -----------------------------------------------------------------------
    // 3. Build Bevy Mesh buffers.
    //    All vertices live on z = 0; normals point toward +Z.
    // -----------------------------------------------------------------------
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(vertex_count);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(vertex_count);

    // Compute AABB for UV mapping
    let (mut min_x, mut max_x) = (f64::MAX, f64::MIN);
    let (mut min_y, mut max_y) = (f64::MAX, f64::MIN);
    for i in 0..vertex_count {
        let vx = flat[i * 2];
        let vy = flat[i * 2 + 1];
        min_x = min_x.min(vx);
        max_x = max_x.max(vx);
        min_y = min_y.min(vy);
        max_y = max_y.max(vy);
    }
    let dx = (max_x - min_x).max(1.0);
    let dy = (max_y - min_y).max(1.0);

    for i in 0..vertex_count {
        let vx = flat[i * 2] as f32;
        let vy = flat[i * 2 + 1] as f32;
        positions.push([vx, vy, 0.0]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([
            ((flat[i * 2] - min_x) / dx) as f32,
            ((flat[i * 2 + 1] - min_y) / dy) as f32,
        ]);
    }

    let indices: Vec<u32> = tri_indices.iter().map(|&i| i as u32).collect();

    // -----------------------------------------------------------------------
    // 4. Assemble and return the Mesh.
    // -----------------------------------------------------------------------
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD);
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        VertexAttributeValues::Float32x3(positions),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_NORMAL,
        VertexAttributeValues::Float32x3(normals),
    );
    mesh.insert_attribute(
        Mesh::ATTRIBUTE_UV_0,
        VertexAttributeValues::Float32x2(uvs),
    );
    mesh.insert_indices(Indices::U32(indices));

    Some(mesh)
}
