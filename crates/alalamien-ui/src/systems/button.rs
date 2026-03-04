use bevy::prelude::*;
use crate::ui::MenuButton;
use crate::audio::MenuAudio;

/// System responsible for button hover visual and audio feedback
/// Follows Single Responsibility Principle: Only handles hover state
pub fn button_hover_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<MenuButton>),
    >,
    mut commands: Commands,
    menu_audio: Res<MenuAudio>,
) {
    for (interaction, mut bg_color) in &mut interaction_query {
        match *interaction {
            Interaction::Hovered => {
                // Ocean blue color (#e94560) when hovered
                *bg_color = BackgroundColor(Color::srgb(0.91, 0.27, 0.38));
                // Play hover sound
                commands.spawn(AudioBundle {
                    source: menu_audio.hover_sound.clone(),
                    ..default()
                });
            }
            Interaction::None => {
                // Dark slate when not hovered
                *bg_color = BackgroundColor(Color::srgb(0.06, 0.21, 0.38));
            }
            _ => {}
        }
    }
}
