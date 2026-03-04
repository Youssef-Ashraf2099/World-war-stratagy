use bevy::prelude::*;
use bevy::winit::WinitWindows;
use bevy::window::PrimaryWindow;

/// Sets the window icon using Bevy's WinitWindows resource.
/// The ICO file is embedded at compile time so asset paths are irrelevant.
pub fn set_window_icon(
    windows: NonSend<WinitWindows>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    let Ok(primary_entity) = primary_window.get_single() else {
        return;
    };
    let Some(winit_window) = windows.get_window(primary_entity) else {
        return;
    };

    // Embed the icon bytes at compile time - no asset path dependency
    let icon_bytes = include_bytes!("../../../assets/images/gameIcon.ico");

    // Decode the ICO file using the image crate
    let img = match image::load_from_memory(icon_bytes) {
        Ok(img) => img.into_rgba8(),
        Err(e) => {
            warn!("Failed to decode gameIcon.ico: {}", e);
            return;
        }
    };

    let (width, height) = img.dimensions();
    let rgba = img.into_raw();

    // Create a winit Icon from the raw RGBA data
    match winit::window::Icon::from_rgba(rgba, width, height) {
        Ok(icon) => {
            winit_window.set_window_icon(Some(icon));
            info!("gameIcon applied to window");
        }
        Err(e) => {
            warn!("Failed to create window icon: {}", e);
        }
    }
}
