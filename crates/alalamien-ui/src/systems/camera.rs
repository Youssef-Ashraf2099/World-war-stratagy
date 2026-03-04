use bevy::input::mouse::{MouseMotion, MouseWheel, MouseScrollUnit};
use bevy::prelude::*;
use crate::map::projection::{MAP_WIDTH, MAP_HEIGHT};

const ZOOM_SPEED: f32 = 0.12;
const ZOOM_MIN: f32 = 0.05;   // very close in
const ZOOM_MAX: f32 = 4.0;    // full world visible
const WASD_PAN_SPEED: f32 = 600.0; // world units per second at scale=1

/// Hard clamp camera so it can never show beyond the map edges.
fn clamp_camera(transform: &mut Transform, proj: &OrthographicProjection) {
    let half_w = MAP_WIDTH  / 2.0;
    let half_h = MAP_HEIGHT / 2.0;
    // At higher zoom (smaller scale) the camera sees more — still keep centre on map
    transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
    transform.translation.y = transform.translation.y.clamp(-half_h, half_h);
    let _ = proj; // available if needed for viewport-aware clamping later
}

/// Pan the camera with right-click / middle-click drag.
pub fn camera_pan(
    mouse_button: Res<ButtonInput<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
    mut camera_q: Query<(&mut Transform, &OrthographicProjection), With<Camera2d>>,
) {
    let is_panning =
        mouse_button.pressed(MouseButton::Right) || mouse_button.pressed(MouseButton::Middle);

    if !is_panning {
        motion_evr.clear();
        return;
    }

    let Ok((mut transform, proj)) = camera_q.get_single_mut() else {
        return;
    };

    for ev in motion_evr.read() {
        // delta.x right = screen right, delta.y down = screen down
        // In world space (Y-up): moving camera right subtracts X, moving up adds Y
        transform.translation.x -= ev.delta.x * proj.scale;
        transform.translation.y += ev.delta.y * proj.scale;
    }

    clamp_camera(&mut transform, proj);
}

/// Pan the camera with WASD / arrow keys.
pub fn camera_wasd(
    keyboard: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut camera_q: Query<(&mut Transform, &OrthographicProjection), With<Camera2d>>,
) {
    let Ok((mut transform, proj)) = camera_q.get_single_mut() else {
        return;
    };

    let speed = WASD_PAN_SPEED * proj.scale * time.delta_seconds();
    let mut delta = Vec2::ZERO;

    if keyboard.pressed(KeyCode::KeyW) || keyboard.pressed(KeyCode::ArrowUp) {
        delta.y += speed;
    }
    if keyboard.pressed(KeyCode::KeyS) || keyboard.pressed(KeyCode::ArrowDown) {
        delta.y -= speed;
    }
    if keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft) {
        delta.x -= speed;
    }
    if keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight) {
        delta.x += speed;
    }

    if delta != Vec2::ZERO {
        transform.translation.x += delta.x;
        transform.translation.y += delta.y;
        clamp_camera(&mut transform, proj);
    }
}

/// Zoom the orthographic projection with the mouse scroll wheel.
/// Zooms toward the cursor position in world space.
pub fn camera_zoom(
    mut scroll_evr: EventReader<MouseWheel>,
    mut camera_q: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
    windows: Query<&Window>,
) {
    let Ok(window) = windows.get_single() else { return };
    let cursor_screen = window.cursor_position();

    for ev in scroll_evr.read() {
        let scroll_y = match ev.unit {
            MouseScrollUnit::Line => ev.y,
            MouseScrollUnit::Pixel => ev.y * 0.05,
        };

        if scroll_y.abs() < 0.001 {
            continue;
        }

        let Ok((mut transform, mut proj)) = camera_q.get_single_mut() else {
            return;
        };

        let old_scale = proj.scale;
        let factor = 1.0 - scroll_y * ZOOM_SPEED;
        proj.scale = (proj.scale * factor).clamp(ZOOM_MIN, ZOOM_MAX);

        // Zoom toward cursor: shift camera so the world point under the cursor stays fixed.
        if let Some(cursor) = cursor_screen {
            let window_size = Vec2::new(window.width(), window.height());
            let ndc = (cursor / window_size) * 2.0 - Vec2::ONE;
            let world_before = transform.translation.truncate()
                + Vec2::new(ndc.x, -ndc.y) * (window_size * 0.5) * old_scale;
            let world_after = transform.translation.truncate()
                + Vec2::new(ndc.x, -ndc.y) * (window_size * 0.5) * proj.scale;
            let correction = world_before - world_after;
            transform.translation.x += correction.x;
            transform.translation.y += correction.y;
        }

        // Always clamp after zoom to prevent edge overshoot
        clamp_camera(&mut transform, &proj);
    }
}

/// Press `F` to fit the whole world into the viewport.
pub fn camera_fit_world(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut camera_q: Query<(&mut Transform, &mut OrthographicProjection), With<Camera2d>>,
    windows: Query<&Window>,
) {
    if !keyboard.just_pressed(KeyCode::KeyF) {
        return;
    }
    let Ok(window) = windows.get_single() else { return };
    let Ok((mut transform, mut proj)) = camera_q.get_single_mut() else { return };

    let scale_x = MAP_WIDTH / window.width();
    let scale_y = MAP_HEIGHT / window.height();
    proj.scale = scale_x.max(scale_y) * 1.05;
    transform.translation = Vec3::ZERO;
}
