use bevy::prelude::*;

#[derive(Component)]
pub struct AnimatedScale {
    pub target_scale: f32,
    pub current_scale: f32,
    pub speed: f32,
}

pub fn animation_system(
    mut query: Query<(&mut AnimatedScale, &mut Transform)>,
    time: Res<Time>,
) {
    for (mut anim, mut transform) in &mut query {
        // Smoothly interpolate scale
        let delta = (anim.target_scale - anim.current_scale).abs();
        if delta > 0.001 {
            anim.current_scale += (anim.target_scale - anim.current_scale)
                * (time.delta_seconds() * anim.speed).min(1.0);
            transform.scale = Vec3::splat(anim.current_scale);
        }
    }
}
