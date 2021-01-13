use iced::widget::button::Style;
use iced::{button, pick_list, Color, Background, Vector};
use iced::futures::prelude::stream::Collect;
use crate::themes::*;
use iced::widget::pick_list::Menu;


pub struct ButtonStyle {
    color: Color,
    is_outlined: bool
}

impl ButtonStyle {
    pub fn new(color: Color, is_outlined: bool) -> Self {
        Self { color, is_outlined }
    }
}

impl button::StyleSheet for ButtonStyle {
    fn active(&self) -> Style {
        button::Style {
            shadow_offset: Default::default(),
            background: if self.is_outlined { DARK_BG.into() } else { self.color.into() },
            border_radius: BUTTON_RADIUS,
            border_width: if self.is_outlined { BUTTON_BORDER_WIDTH } else { 0.0 },
            border_color: if self.is_outlined { self.color.into() } else { WHITE.into() },
            text_color: if self.is_outlined { self.color.into() } else { WHITE.into() },
        }
    }

    fn hovered(&self) -> Style {
        button::Style {
            shadow_offset: Vector::new(0.0, 1.0),
            background: if self.is_outlined { DARK_BG.into() } else { self.color.into() },
            border_radius: BUTTON_RADIUS,
            border_width: if self.is_outlined { BUTTON_BORDER_WIDTH } else { 0.0 },
            border_color: if self.is_outlined { self.color.into() } else { WHITE.into() },
            text_color: if self.is_outlined { self.color.into() } else { WHITE.into() },
        }
    }

    fn pressed(&self) -> Style {
        self.active()
    }

    fn disabled(&self) -> Style {
        button::Style {
            shadow_offset: Default::default(),
            background: if self.is_outlined { DARK_BG.into() } else { GREY.into() },
            border_radius: BUTTON_RADIUS,
            border_width: if self.is_outlined { BUTTON_BORDER_WIDTH } else { 0.0 },
            border_color: if self.is_outlined { GREY.into() } else { WHITE.into() },
            text_color: if self.is_outlined { GREY.into() } else { WHITE.into() },
        }
    }
}

pub struct DropDown;

impl pick_list::StyleSheet for DropDown {
    fn menu(&self) -> Menu {
        Menu {
            text_color: WHITE.into(),
            background: DARK_BG.into(),
            border_width: BUTTON_RADIUS,
            border_color: WHITE.into(),
            selected_text_color: WHITE.into(),
            selected_background: GREY.into()
        }
    }

    fn active(&self) -> pick_list::Style {
        pick_list::Style {
            text_color: WHITE.into(),
            background: DARK_BG.into(),
            border_radius: BUTTON_RADIUS,
            border_width: 1.0,
            border_color: WHITE.into(),
            icon_size: 0.5
        }
    }

    fn hovered(&self) -> pick_list::Style {
        pick_list::Style {
            text_color: WHITE.into(),
            background: GREY.into(),
            border_radius: BUTTON_RADIUS,
            border_width: 1.0,
            border_color: WHITE.into(),
            icon_size: 0.5
        }
    }
}

pub struct Container;
impl iced::container::StyleSheet for Container {
    fn style(&self) -> iced::container::Style {
        iced::container::Style {
            text_color: WHITE.into(),
            background: DARK_BG.into(),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Default::default()
        }
    }
}
