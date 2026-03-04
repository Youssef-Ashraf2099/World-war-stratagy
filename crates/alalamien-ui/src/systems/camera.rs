use bevy::input::mouse::{MouseMotion, MouseWheel, MouseScrollUnit};
use bevy::prelude::*;
use crate::map::projection::{MAP_WIDTH, MAP_HEIGHT};

const ZOOM_SPEED: f32 = 0.12;
const ZOOM_MIN: f32 = 0.05;   // most zoomed-in scale allowed
const WASD_PAN_SPEED: f32 = 600.0; // world units per second at scale=1

/// Clamp the camera so the **viewport edges** never go beyond the map boundary.
/// `vp_half` is (window_width/2, window_height/2) in pixels — the camera sees
/// `vp_half * scale` world-units from its centre in each direction.
/// If the map is smaller than the viewport (very zoomed-out) we just centre it.
fn clamp_camera(transform: &mut Transform, scale: f32, vp_half: Vec2) {
    let map_half = Vec2::new(MAP_WIDTH / 2.0, MAP_HEIGHT / 2.0);
    let seen = vp_half * scale; // world-units visible from centre to edge

    // If the map fits entirely in the viewport on an axis, centre on that axis.
    // Otherwise restrict the centre so the viewport edge stays inside the map.
    let clamp_x = if seen.x >= map_half.x {
        0.0
    } else {
        transform.translation.x.clamp(-(map_half.x - seen.x), map_half.x - seen.x)
    };
    let clamp_y = if seen.y >= map_half.y {
        0.0
    } else {
        transform.translation.y.clamp(-(map_half.y - seen.y), map_half.y - seen.y)
    };

    transform.translation.x = clamp_x;
    transform.translation.y = clamp_y;
}

/// Called once on `OnEnter(AppState::Game)` — fits the full world into view
/// so the player starts with no grey void visible.
pub fn reset_camera_for_game(
    mut camera_q: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return };
    let Ok((mut transform, mut proj)) = camera_q.get_single_mut() else { return };

    let scale_x = MAP_WIDTH  / window.width();
    let scale_y = MAP_HEIGHT / window.height();
    proj.scale = scale_x.max(scale_y); // exactly fits — no padding so no void
    transform.translation = Vec3::ZERO;
}

/// Pan the camera with right-click / middle-click drag.
pub fn camera_pan(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut camera_q: Query<(&mut Transform, &OrthographicProjection), With<Camera2d>>,
    windows: Query<&Window>,
) {
    let is_panning =
        mouse_button.pressed(MouseButton::Right) || mouse_button.pressed(MouseButton::Middle);

    if !is_panning {
        motion_evr.clear();
        return;
    }

    let Ok(window) = windows.get_single() else { return };
    let vp_half = Vec2::new(window.width() / 2.0, window.height() / 2.0);
    let Ok((mut transform, proj)) = camera_q.get_single_mut() else { return };
    let scale = proj.scale;

    for ev in motion_evr.read() {
        transform.translation.x -= ev.delta.x * scale;
        transform.translation.y += ev.delta.y * scale;
    }

    clamp_camera(&mut transform, scale, vp_half);
}

/// Pan the camera with WASD / arrow keys.
pub fn camera_wasd(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_q: Query<(&mut Transform, &OrthographicProjection), With<Camera2d>>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return };
    let vp_half = Vec2::new(window.width() / 2.0, window.height() / 2.0);
    let Ok((mut transform, proj)) = camera_q.get_single_mut() else { return };
    let scale = proj.scale;
    let speed = WASD_PAN_SPEED * scale * time.delta_seconds();

    let mut delta = Vec2::ZERO;
    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp)    { delta.y += speed; }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown)  { delta.y -= speed; }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft)  { delta.x -= speed; }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) { delta.x += speed; }

    if delta != Vec2::ZERO {
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
        clamp_camera(&mut transform, scale, vp_half);
    }
}

/// Zoom the orthographic projection with the mouse scroll wheel,
/// zooming toward the cursor and always clamping to keep the map full.
pub fn camera_zoom(
    mut scroll_evr: EventReader<MouseWheel>,
    mut camera_q: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return };
    let vp_half = Vec2::new(window.width() / 2.0, window.height() / 2.0);

    // The minimum scale at which the full map is still inside the viewport.
    let min_scale_x = MAP_WIDTH  / window.width();
    let min_scale_y = MAP_HEIGHT / window.height();
    let zoom_floor  = min_scale_x.max(min_scale_y); // never zoom out past fit-world

    let cursor_screen = window.cursor_position();

    for ev in scroll_evr.read() {
        let scroll_y = match ev.unit {
            MouseScrollUnit::Line  => ev.y,
            MouseScrollUnit::Pixel => ev.y * 0.05,
        };
        if scroll_y.abs() < 0.001 { continue; }

        let Ok((mut transform, mut proj)) = camera_q.get_single_mut() else { return };
        let old_scale = proj.scale;
        // ZOOM_MIN  = most zoomed-in  (smallest scale)
        // zoom_floor = most zoomed-out (largest scale = full world visible, no void)
        proj.scale = (proj.scale * (1.0 - scroll_y * ZOOM_SPEED)).clamp(ZOOM_MIN, zoom_floor);
        let new_scale = proj.scale;

        // Zoom toward cursor
        if let Some(cursor) = cursor_screen {
            let window_size = Vec2::new(window.width(), window.height());
            let ndc = (cursor / window_size) * 2.0 - Vec2::ONE;
            let world_before = transform.translation.truncate()
                + Vec2::new(ndc.x, -ndc.y) * vp_half * old_scale;
            let world_after  = transform.translation.truncate()
                + Vec2::new(ndc.x, -ndc.y) * vp_half * new_scale;
            let correction = world_before - world_after;
            transform.translation.x += correction.x;
            transform.translation.y += correction.y;
        }

        clamp_camera(&mut transform, new_scale, vp_half);
    }
}

/// Press `F` to fit the whole world into the viewport (same as initial view).
pub fn camera_fit_world(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
    windows: Query<&Window>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) { return; }
    let Ok(window) = windows.get_single() else { return };
    let Ok((mut transform, mut proj)) = camera_q.get_single_mut() else { return };

    let scale_x = MAP_WIDTH  / window.width();
    let scale_y = MAP_HEIGHT / window.height();
    proj.scale = scale_x.max(scale_y);
    transform.translation = Vec3::ZERO;
}
