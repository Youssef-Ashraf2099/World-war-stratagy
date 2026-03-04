use bevy::prelude::*;
use bevy::sprite::ColorMaterial;
use crate::map::nation_colors::{color_for_nation, hover_color, selected_color};

// ---------------------------------------------------------------------------
// Components
// ---------------------------------------------------------------------------

/// Marks a mesh entity as belonging to a renderable nation polygon.
#[derive(Component, Clone)]
pub struct NationMesh {
    pub nation_index: usize,
    pub iso_a3: String,
    pub name: String,
    /// Base colour (un-highlighted).
    pub base_color: Color,
    /// World-space AABB [min_x, min_y, max_x, max_y] for fast cursor picking.
    pub aabb: [f32; 4],
}

// ---------------------------------------------------------------------------
// Resources
// ---------------------------------------------------------------------------

/// Currently-hovered nation (if any).
#[derive(Resource, Default)]
pub struct HoveredNation {
    pub iso_a3: Option<String>,
    pub name: Option<String>,
}

/// Currently-selected nation (if any).
#[derive(Resource, Default)]
pub struct SelectedNation {
    pub iso_a3: Option<String>,
    pub name: Option<String>,
    pub nation_index: Option<usize>,
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

/// Update [`HoveredNation`] and mesh colours based on cursor world position.
pub fn hover_system(
    windows: Query<&Window>,
    camera_q: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut nation_meshes: Query<(&NationMesh, &mut Handle<ColorMaterial>)>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut hovered: ResMut<HoveredNation>,
    selected: Res<SelectedNation>,
) {
    let Ok(window) = windows.get_single() else { return };
    let Ok((camera, cam_transform)) = camera_q.get_single() else { return };

    // Convert cursor position to world coordinates.
    let world_pos = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(cam_transform, cursor));

    let cursor_world = match world_pos {
        Some(p) => p,
        None => {
            // No cursor on screen — clear hover
            if hovered.iso_a3.is_some() {
                hovered.iso_a3 = None;
                hovered.name = None;
                // Restore colours
                recolor_all(&mut nation_meshes, &mut materials, &selected);
            }
            return;
        }
    };

    // Determine which nation (if any) the cursor is over via AABB.
    // Among all matching AABBs pick the one with the SMALLEST area so that
    // small nations (Luxembourg, Vatican) beat their large neighbours (France, Italy).
    let mut best_iso: Option<String> = None;
    let mut best_name: Option<String> = None;
    let mut best_area = f32::MAX;

    for (nm, _) in &nation_meshes {
        let [min_x, min_y, max_x, max_y] = nm.aabb;
        if cursor_world.x >= min_x
            && cursor_world.x <= max_x
            && cursor_world.y >= min_y
            && cursor_world.y <= max_y
        {
            let area = (max_x - min_x) * (max_y - min_y);
            if area < best_area {
                best_area = area;
                best_iso = Some(nm.iso_a3.clone());
                best_name = Some(nm.name.clone());
            }
        }
    }
    let new_hover = best_iso;
    let new_name = best_name;

    if new_hover != hovered.iso_a3 {
        hovered.iso_a3 = new_hover;
        hovered.name = new_name;
        recolor_all(&mut nation_meshes, &mut materials, &selected);
        // Apply hover highlight
        if let Some(ref iso) = hovered.iso_a3 {
            for (nm, mat_handle) in &mut nation_meshes {
                if &nm.iso_a3 == iso {
                    if let Some(mat) = materials.get_mut(mat_handle.id()) {
                        mat.color = hover_color(nm.base_color);
                    }
                }
            }
        }
    }
}

/// Handle left-click to select a nation.
pub fn click_select_system(
    mouse: Res<ButtonInput<MouseButton>>,
    hovered: Res<HoveredNation>,
    mut selected: ResMut<SelectedNation>,
    nation_meshes: Query<&NationMesh>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mesh_q: Query<(&NationMesh, &Handle<ColorMaterial>)>,
) {
    if !mouse.just_pressed(MouseButton::Left) {
        return;
    }

    let prev = selected.iso_a3.clone();

    if let Some(ref iso) = hovered.iso_a3.clone() {
        // Find index
        let entry = nation_meshes.iter().find(|nm| &nm.iso_a3 == iso);
        if let Some(nm) = entry {
            selected.iso_a3 = Some(nm.iso_a3.clone());
            selected.name = Some(nm.name.clone());
            selected.nation_index = Some(nm.nation_index);
        }
    } else {
        // Clicked empty space — deselect
        selected.iso_a3 = None;
        selected.name = None;
        selected.nation_index = None;
    }

    if selected.iso_a3 != prev {
        // Re-apply colours: restore previous, highlight new
        for (nm, mat_handle) in &mesh_q {
            if let Some(mat) = materials.get_mut(mat_handle.id()) {
                if selected.iso_a3.as_deref() == Some(&nm.iso_a3) {
                    mat.color = selected_color();
                } else {
                    mat.color = nm.base_color;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn recolor_all(
    meshes: &mut Query<(&NationMesh, &mut Handle<ColorMaterial>)>,
    materials: &mut Assets<ColorMaterial>,
    selected: &SelectedNation,
) {
    for (nm, mat_handle) in meshes.iter_mut() {
        if let Some(mat) = materials.get_mut(mat_handle.id()) {
            if selected.iso_a3.as_deref() == Some(&nm.iso_a3) {
                mat.color = selected_color();
            } else {
                mat.color = nm.base_color;
            }
        }
    }
}

/// Compute an AABB from a polygon's group outer ring + holes.
pub fn compute_aabb(vertices: &[(f32, f32)]) -> [f32; 4] {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for &(x, y) in vertices {
        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x);
        max_y = max_y.max(y);
    }
    [min_x, min_y, max_x, max_y]
}

/// Compute a nation-level AABB spanning all its polygon groups' outer rings.
pub fn nation_aabb(groups: &[crate::map::shapefile_loader::PolygonGroupData]) -> [f32; 4] {
    let mut min_x = f32::MAX;
    let mut min_y = f32::MAX;
    let mut max_x = f32::MIN;
    let mut max_y = f32::MIN;
    for g in groups {
        let [a, b, c, d] = compute_aabb(&g.outer);
        min_x = min_x.min(a);
        min_y = min_y.min(b);
        max_x = max_x.max(c);
        max_y = max_y.max(d);
    }
    [min_x, min_y, max_x, max_y]
}

/// Initialise `color_for_nation` lookup — exported so `map_plugin` can call it.
#[allow(dead_code)]
pub fn nation_base_color(iso_a3: &str) -> Color {
    color_for_nation(iso_a3)
}
