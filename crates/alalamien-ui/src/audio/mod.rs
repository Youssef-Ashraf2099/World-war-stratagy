use bevy::prelude::*;

#[derive(Resource)]
pub struct MenuAudio {
    pub click_sound: Handle<AudioSource>,
    pub hover_sound: Handle<AudioSource>,
}

pub fn load_menu_audio(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let click_sound = asset_server.load("sounds/click.mp3");
    let hover_sound = asset_server.load("sounds/hover.mp3");

    commands.insert_resource(MenuAudio {
        click_sound,
        hover_sound,
    });
}
