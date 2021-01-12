use iced::widget::button::Style;
use iced::{button, Color, Background};
use iced::futures::prelude::stream::Collect;
use crate::themes::*;

pub struct MaterialButton;

impl button::StyleSheet for MaterialButton {
    fn active(&self) -> Style {
        button::Style {
            shadow_offset: Default::default(),
            background: hex_to_color(0x5C, 0x13, 0x49).into(),
            border_radius: BUTTON_RADIUS,
            border_width: Default::default(),
            border_color: Default::default(),
            text_color: Color::from_rgb(255.0,255.0,255.0)
        }
    }

    fn hovered(&self) -> Style {
        button::Style {
            shadow_offset: Default::default(),
            background: hex_to_color(0x5C, 0x13, 0x49).into(),
            border_radius: BUTTON_RADIUS,
            border_width: Default::default(),
            border_color: Default::default(),
            text_color: Color::from_rgb(255.0,255.0,255.0)
        }
    }

    fn pressed(&self) -> Style {
        button::Style {
            shadow_offset: Default::default(),
            background: hex_to_color(0x5C, 0x13, 0x49).into(),
            border_radius: BUTTON_RADIUS,
            border_width: Default::default(),
            border_color: Default::default(),
            text_color: Color::from_rgb(255.0,255.0,255.0)
        }
    }

    fn disabled(&self) -> Style {
        button::Style {
            shadow_offset: Default::default(),
            background: Some(Background::Color(Color::from_rgb(92.0,19.0,73.0))),
            border_radius: BUTTON_RADIUS,
            border_width: Default::default(),
            border_color: Default::default(),
            text_color: Color::from_rgb(255.0,255.0,255.0)
        }
    }
}