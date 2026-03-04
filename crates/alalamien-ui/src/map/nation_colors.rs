use bevy::prelude::Color;

/// A hand-curated palette of 20 distinct, visually-pleasing political-map colours.
/// Nations are assigned by hashing their ISO-A3 code so the assignment is
/// deterministic but varied across neighbouring nations.
const PALETTE: &[(f32, f32, f32)] = &[
    (0.47, 0.67, 0.47), // muted green
    (0.65, 0.50, 0.35), // tan / sand
    (0.40, 0.55, 0.70), // steel blue
    (0.72, 0.46, 0.46), // rose
    (0.58, 0.72, 0.58), // sage green
    (0.55, 0.45, 0.65), // mauve
    (0.70, 0.65, 0.40), // olive gold
    (0.45, 0.60, 0.65), // teal
    (0.78, 0.60, 0.45), // warm ochre
    (0.50, 0.68, 0.60), // aqua green
    (0.60, 0.42, 0.50), // dusty rose
    (0.48, 0.62, 0.78), // periwinkle
    (0.70, 0.70, 0.45), // mustard
    (0.42, 0.50, 0.58), // slate
    (0.65, 0.55, 0.40), // caramel
    (0.52, 0.72, 0.52), // light green
    (0.62, 0.42, 0.62), // purple
    (0.68, 0.63, 0.52), // khaki
    (0.45, 0.65, 0.55), // jungle green
    (0.76, 0.55, 0.36), // burnt orange
];

/// Derive a palette colour for `iso_a3` by simple string hash.
/// Returns a `Color::srgb(r, g, b)` value.
pub fn color_for_nation(iso_a3: &str) -> Color {
    let hash: usize = iso_a3
        .bytes()
        .enumerate()
        .fold(0usize, |acc, (i, b)| {
            acc.wrapping_add((b as usize).wrapping_mul(31usize.wrapping_pow(i as u32)))
        });
    let (r, g, b) = PALETTE[hash % PALETTE.len()];
    Color::srgb(r, g, b)
}

/// Highlighted colour for a hovered nation (slightly brightened).
pub fn hover_color(base: Color) -> Color {
    let lch = base.to_linear();
    Color::linear_rgb(
        (lch.red * 1.35).min(1.0),
        (lch.green * 1.35).min(1.0),
        (lch.blue * 1.35).min(1.0),
    )
}

/// Colour used when a nation is selected.
pub fn selected_color() -> Color {
    Color::srgb(1.0, 0.85, 0.2) // golden yellow
}
