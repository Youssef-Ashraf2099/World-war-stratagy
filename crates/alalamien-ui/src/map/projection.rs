/// Equirectangular (plate carrée) projection utilities
/// World map is rendered in a 4096 × 2048 unit coordinate space
/// centred on (0, 0) — matching Bevy's default Camera2d origin.
///
///   lon -180..180  →  x  -2048..2048
///   lat  -90.. 90  →  y  -1024..1024   (Bevy Y-up matches geographic N-up)

pub const MAP_WIDTH: f32 = 4096.0;
pub const MAP_HEIGHT: f32 = 2048.0;

/// Convert geographic longitude to world-space X.
#[inline]
pub fn lon_to_x(lon: f64) -> f32 {
    (lon / 180.0 * (MAP_WIDTH as f64 / 2.0)) as f32
}

/// Convert geographic latitude to world-space Y.
/// Latitude increases northward, as does Bevy's Y-axis — no flip needed.
#[inline]
pub fn lat_to_y(lat: f64) -> f32 {
    (lat / 90.0 * (MAP_HEIGHT as f64 / 2.0)) as f32
}

/// Convert a world-space position back to (lon, lat).
#[allow(dead_code)]
#[inline]
pub fn world_to_lonlat(x: f32, y: f32) -> (f64, f64) {
    let lon = (x as f64) / (MAP_WIDTH as f64 / 2.0) * 180.0;
    let lat = (y as f64) / (MAP_HEIGHT as f64 / 2.0) * 90.0;
    (lon, lat)
}
