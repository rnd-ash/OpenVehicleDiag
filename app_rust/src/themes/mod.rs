pub mod dark;
pub mod light;

use iced::{button};

const BUTTON_RADIUS : f32 = 2.0;
const BUTTON_MIN_WIDTH:f32 = 64.0;
const BUTTON_MIN_HEIGHT: f32 = 36.0;

pub enum Style {
    Light,
    Dark
}