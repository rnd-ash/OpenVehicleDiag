use iced::widget::button::Style;
use iced::{button, Color};
use iced::futures::prelude::stream::Collect;
use crate::themes::*;

pub struct MaterialButton;

impl button::StyleSheet for MaterialButton {
    fn active(&self) -> Style {
        button::Style {
            shadow_offset: Default::default(),
            background: Some(Color::from_rgb(92.0,19.0,73.0).into()),
            border_radius: BUTTON_RADIUS,
            border_width: BUTTON_RADIUS,
            border_color: Default::default(),
            text_color: Color::from_rgb(255.0,255.0,255.0)
        }
    }

    fn hovered(&self) -> Style {
        unimplemented!()
    }

    fn pressed(&self) -> Style {
        unimplemented!()
    }

    fn disabled(&self) -> Style {
        unimplemented!()
    }
}