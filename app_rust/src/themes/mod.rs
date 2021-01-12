pub mod dark;
pub mod light;

use iced::{button, Color};

const BUTTON_RADIUS : f32 = 3.0;
const BUTTON_MIN_WIDTH:f32 = 64.0;
const BUTTON_MIN_HEIGHT: f32 = 36.0;

pub enum Style {
    Light,
    Dark
}

pub (crate) fn hex_to_color(r: u8, g: u8, b: u8) -> Color {
    return Color::from_rgb(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
    )
}