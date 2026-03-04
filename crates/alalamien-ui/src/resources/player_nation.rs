use bevy::prelude::*;

/// The nation the player has chosen to control.
/// `None` means no nation has been selected yet.
#[derive(Resource, Default, Clone)]
pub struct PlayerNation {
    pub iso_a3: Option<String>,
    pub name: Option<String>,
}
