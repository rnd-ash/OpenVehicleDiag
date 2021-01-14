use iced::widget::button::Style;
use iced::{button, pick_list, Color, Vector};
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
        match super::get_theme() {
            super::Style::Light => button::Style {
                shadow_offset: Default::default(),
                background: if self.is_outlined { WHITE.into() } else { self.color.into() },
                border_radius: BUTTON_RADIUS,
                border_width: if self.is_outlined { BUTTON_BORDER_WIDTH } else { 0.0 },
                border_color: if self.is_outlined { self.color.into() } else { WHITE.into() },
                text_color: if self.is_outlined { self.color.into() } else { WHITE.into() },
            },
            super::Style::Dark => button::Style {
                shadow_offset: Default::default(),
                background: if self.is_outlined { DARK_BG.into() } else { self.color.into() },
                border_radius: BUTTON_RADIUS,
                border_width: if self.is_outlined { BUTTON_BORDER_WIDTH } else { 0.0 },
                border_color: if self.is_outlined { self.color.into() } else { WHITE.into() },
                text_color: if self.is_outlined { self.color.into() } else { WHITE.into() },
            }
        }
    }

    fn hovered(&self) -> Style {
        Style {
            shadow_offset: Vector::new(0.0, 1.0),
            ..self.active()
        }
    }

    fn pressed(&self) -> Style {
        self.active()
    }

    fn disabled(&self) -> Style {
        match super::get_theme() {
            super::Style::Light => button::Style {
                background: if self.is_outlined { WHITE.into() } else { self.color.into() },
                text_color: GREY.into(),
                border_color: GREY.into(),
                ..self.active()
            },
            super::Style::Dark => button::Style {
                background: if self.is_outlined { DARK_BG.into() } else { self.color.into() },
                text_color: GREY.into(),
                border_color: GREY.into(),
                ..self.active()
            }
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
        match super::get_theme() {
            super::Style::Light => pick_list::Style {
                text_color: DARK_BG.into(),
                background: WHITE.into(),
                border_radius: BUTTON_RADIUS,
                border_width: 1.0,
                border_color: DARK_BG.into(),
                icon_size: 0.5
            },
            super::Style::Dark => pick_list::Style {
                text_color: WHITE.into(),
                background: DARK_BG.into(),
                border_radius: BUTTON_RADIUS,
                border_width: 1.0,
                border_color: WHITE.into(),
                icon_size: 0.5
            }
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
        match super::get_theme() {
            super::Style::Light => iced::container::Style {
                text_color: DARK_BG.into(),
                background: WHITE.into(),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Default::default()
            },
            super::Style::Dark => iced::container::Style {
                text_color: WHITE.into(),
                background: DARK_BG.into(),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Default::default()
            }
        }
    }
}

pub struct RadioBtn {
    c: Color
}
impl RadioBtn {
    pub (crate) fn new(style: ButtonType) -> Self {
        Self { c: style.get_colour() }
    }
}

impl iced::radio::StyleSheet for RadioBtn {
    fn active(&self) -> iced::radio::Style {
        match super::get_theme() {
            super::Style::Light => iced::radio::Style {
                background: WHITE.into(),
                dot_color: self.c.into(),
                border_width: 1.0,
                border_color: DARK_BG.into()
            },
            super::Style::Dark => iced::radio::Style {
                background: DARK_BG.into(),
                dot_color: self.c.into(),
                border_width: 1.0,
                border_color: WHITE.into()
            }
        }
    }

    fn hovered(&self) -> iced::radio::Style {
        match super::get_theme() {
            super::Style::Light => iced::radio::Style {
                background: WHITE.into(),
                dot_color: self.c.into(),
                border_width: 1.0,
                border_color: WHITE.into()
            },
            super::Style::Dark => iced::radio::Style {
                background: DARK_BG.into(),
                dot_color: WHITE.into(),
                border_width: 1.0,
                border_color: WHITE.into()
            }
        }
    }
}

pub struct PBar {
    c: Color
}
impl PBar {
    pub fn new(accent: ButtonType) -> Self {
        Self { c: accent.get_colour() }
    }
}

impl iced::progress_bar::StyleSheet for PBar {
    fn style(&self) -> iced::progress_bar::Style {
        iced::progress_bar::Style {
            background: GREY.into(),
            bar: self.c.into(),
            border_radius: BUTTON_RADIUS
        }
    }
}
