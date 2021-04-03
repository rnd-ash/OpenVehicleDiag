use crate::themes::*;
use iced::widget::button::Style;
use iced::widget::pick_list::Menu;
use iced::{button, pick_list, Color, Vector};

pub struct ButtonStyle {
    color: Color,
    is_outlined: bool,
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
                background: if self.is_outlined {
                    WHITE.into()
                } else {
                    self.color.into()
                },
                border_radius: BUTTON_RADIUS,
                border_width: if self.is_outlined {
                    BUTTON_BORDER_WIDTH
                } else {
                    0.0
                },
                border_color: if self.is_outlined { self.color } else { WHITE },
                text_color: if self.is_outlined { self.color } else { WHITE },
            },
            super::Style::Dark => button::Style {
                shadow_offset: Default::default(),
                background: if self.is_outlined {
                    DARK_BG.into()
                } else {
                    self.color.into()
                },
                border_radius: BUTTON_RADIUS,
                border_width: if self.is_outlined {
                    BUTTON_BORDER_WIDTH
                } else {
                    0.0
                },
                border_color: if self.is_outlined { self.color } else { WHITE },
                text_color: if self.is_outlined { self.color } else { WHITE },
            },
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
                background: if self.is_outlined {
                    WHITE.into()
                } else {
                    GREY.into()
                },
                text_color: if self.is_outlined { GREY } else { WHITE },
                border_color: if self.is_outlined { GREY } else { WHITE },
                ..self.active()
            },
            super::Style::Dark => button::Style {
                background: if self.is_outlined {
                    DARK_BG.into()
                } else {
                    GREY.into()
                },
                text_color: if self.is_outlined { GREY } else { WHITE },
                border_color: if self.is_outlined { GREY } else { WHITE },
                ..self.active()
            },
        }
    }
}


pub struct ButtonTableStyle {
    color: Color,
    selected: bool,
}

impl ButtonTableStyle {
    pub fn new(color: Color, selected: bool) -> Self {
        Self { color, selected }
    }
}

impl button::StyleSheet for ButtonTableStyle {
    fn active(&self) -> Style {
        if self.selected {
            match super::get_theme() {
                super::Style::Light => button::Style {
                    shadow_offset: Default::default(),
                    background: GREY.into(),
                    border_radius: 0f32,
                    border_width: 0f32,
                    border_color: WHITE,
                    text_color: self.color,
                    
                },
                super::Style::Dark => button::Style {
                    shadow_offset: Default::default(),
                    background: GREY.into(),
                    border_radius: 0f32,
                    border_width: 0f32,
                    border_color: DARK_BG,
                    text_color: self.color,
                },
            }
        } else {
            match super::get_theme() {
                super::Style::Light => button::Style {
                    shadow_offset: Default::default(),
                    background: WHITE.into(),
                    border_radius: 0f32,
                    border_width: 0f32,
                    border_color: WHITE,
                    text_color: self.color,
                    
                },
                super::Style::Dark => button::Style {
                    shadow_offset: Default::default(),
                    background: DARK_BG.into(),
                    border_radius: 0f32,
                    border_width: 0f32,
                    border_color: DARK_BG,
                    text_color: self.color,
                },
            }
        }
    }

    fn hovered(&self) -> Style {
        match super::get_theme() {
            super::Style::Light => button::Style {
                shadow_offset: Default::default(),
                background: if self.selected { GREY.into() } else { WHITE.into() },
                border_radius: 0f32,
                border_width: 0f32,
                border_color: WHITE,
                text_color: DARK_BG,
                
            },
            super::Style::Dark => button::Style {
                shadow_offset: Default::default(),
                background: if self.selected { GREY.into() } else { DARK_BG.into() },
                border_radius: 0f32,
                border_width: 0f32,
                border_color: DARK_BG,
                text_color: WHITE,
            },
        }
    }

    fn pressed(&self) -> Style {
        self.active()
    }

    fn disabled(&self) -> Style {
        match super::get_theme() {
            super::Style::Light => button::Style {
                shadow_offset: Default::default(),
                background: WHITE.into(),
                border_radius: 0f32,
                border_width: 0f32,
                border_color: WHITE,
                text_color: GREY,
                
            },
            super::Style::Dark => button::Style {
                shadow_offset: Default::default(),
                background: DARK_BG.into(),
                border_radius: 0f32,
                border_width: 0f32,
                border_color: DARK_BG,
                text_color: GREY,
            },
        }
    }
}

pub struct DropDown;

impl pick_list::StyleSheet for DropDown {
    fn menu(&self) -> Menu {
        match super::get_theme() {
            super::Style::Light => Menu {
                text_color: DARK_BG,
                background: WHITE.into(),
                border_width: 1.0,
                border_color: DARK_BG,
                selected_text_color: DARK_BG,
                selected_background: GREY.into(),
            },
            super::Style::Dark => Menu {
                text_color: WHITE,
                background: DARK_BG.into(),
                border_width: 1.0,
                border_color: WHITE,
                selected_text_color: WHITE,
                selected_background: GREY.into(),
            },
        }
    }

    fn active(&self) -> pick_list::Style {
        match super::get_theme() {
            super::Style::Light => pick_list::Style {
                text_color: DARK_BG,
                background: WHITE.into(),
                border_radius: BUTTON_RADIUS,
                border_width: 1.0,
                border_color: DARK_BG,
                icon_size: 0.5,
            },
            super::Style::Dark => pick_list::Style {
                text_color: WHITE,
                background: DARK_BG.into(),
                border_radius: BUTTON_RADIUS,
                border_width: 1.0,
                border_color: WHITE,
                icon_size: 0.5,
            },
        }
    }

    fn hovered(&self) -> pick_list::Style {
        pick_list::Style {
            text_color: WHITE,
            background: GREY.into(),
            border_radius: BUTTON_RADIUS,
            border_width: 1.0,
            border_color: WHITE,
            icon_size: 0.5,
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
                border_color: Default::default(),
            },
            super::Style::Dark => iced::container::Style {
                text_color: WHITE.into(),
                background: DARK_BG.into(),
                border_radius: 0.0,
                border_width: 0.0,
                border_color: Default::default(),
            },
        }
    }
}

pub struct RadioBtn {
    c: Color,
}
impl RadioBtn {
    pub(crate) fn new(style: ButtonType) -> Self {
        Self {
            c: style.get_colour(),
        }
    }
}

impl iced::radio::StyleSheet for RadioBtn {
    fn active(&self) -> iced::radio::Style {
        match super::get_theme() {
            super::Style::Light => iced::radio::Style {
                background: WHITE.into(),
                dot_color: self.c,
                border_width: 1.0,
                border_color: DARK_BG,
            },
            super::Style::Dark => iced::radio::Style {
                background: DARK_BG.into(),
                dot_color: self.c,
                border_width: 1.0,
                border_color: WHITE,
            },
        }
    }

    fn hovered(&self) -> iced::radio::Style {
        match super::get_theme() {
            super::Style::Light => iced::radio::Style {
                background: WHITE.into(),
                dot_color: self.c,
                border_width: 1.0,
                border_color: WHITE,
            },
            super::Style::Dark => iced::radio::Style {
                background: DARK_BG.into(),
                dot_color: WHITE,
                border_width: 1.0,
                border_color: WHITE,
            },
        }
    }
}

pub struct PBar {
    c: Color,
}
impl PBar {
    pub fn new(accent: ButtonType) -> Self {
        Self {
            c: accent.get_colour(),
        }
    }
}

impl iced::progress_bar::StyleSheet for PBar {
    fn style(&self) -> iced::progress_bar::Style {
        iced::progress_bar::Style {
            background: GREY.into(),
            bar: self.c.into(),
            border_radius: BUTTON_RADIUS,
        }
    }
}

pub struct TextInput;

impl iced::text_input::StyleSheet for TextInput {
    fn active(&self) -> iced::text_input::Style {
        match super::get_theme() {
            super::Style::Light => iced::text_input::Style {
                background: WHITE.into(),
                border_radius: BUTTON_RADIUS,
                border_width: BUTTON_BORDER_WIDTH,
                border_color: GREY,
            },
            super::Style::Dark => iced::text_input::Style {
                background: DARK_BG.into(),
                border_radius: BUTTON_RADIUS,
                border_width: BUTTON_BORDER_WIDTH,
                border_color: WHITE,
            },
        }
    }

    fn focused(&self) -> iced::text_input::Style {
        match super::get_theme() {
            super::Style::Light => iced::text_input::Style {
                background: WHITE.into(),
                border_radius: BUTTON_RADIUS,
                border_width: BUTTON_BORDER_WIDTH,
                border_color: GREY,
            },
            super::Style::Dark => iced::text_input::Style {
                background: DARK_BG.into(),
                border_radius: BUTTON_RADIUS,
                border_width: BUTTON_BORDER_WIDTH,
                border_color: WHITE,
            },
        }
    }

    fn placeholder_color(&self) -> Color {
        match super::get_theme() {
            super::Style::Light => super::GREY,
            super::Style::Dark => super::WHITE,
        }
    }

    fn value_color(&self) -> Color {
        match super::get_theme() {
            super::Style::Light => Color::BLACK,
            super::Style::Dark => super::WHITE,
        }
    }

    fn selection_color(&self) -> Color {
        match super::get_theme() {
            super::Style::Light => super::GREY,
            super::Style::Dark => super::WHITE,
        }
    }
}

pub struct CheckBox;
impl iced::checkbox::StyleSheet for CheckBox {
    fn active(&self, is_checked: bool) -> iced::checkbox::Style {
        match super::get_theme() {
            super::Style::Light => iced::checkbox::Style {
                background: super::WHITE.into(),
                checkmark_color: super::DARK_BG,
                border_radius: BUTTON_RADIUS,
                border_width: BUTTON_BORDER_WIDTH,
                border_color: super::DARK_BG,
            },
            super::Style::Dark => iced::checkbox::Style {
                background: super::DARK_BG.into(),
                checkmark_color: super::WHITE,
                border_radius: BUTTON_RADIUS,
                border_width: BUTTON_BORDER_WIDTH,
                border_color: super::WHITE,
            }
        }
    }

    fn hovered(&self, is_checked: bool) -> iced::checkbox::Style {
        match super::get_theme() {
            super::Style::Light => iced::checkbox::Style {
                background: super::WHITE.into(),
                checkmark_color: super::DARK_BG,
                border_radius: BUTTON_RADIUS,
                border_width: BUTTON_BORDER_WIDTH,
                border_color: super::DARK_BG,
            },
            super::Style::Dark => iced::checkbox::Style {
                background: super::DARK_BG.into(),
                checkmark_color: super::WHITE,
                border_radius: BUTTON_RADIUS,
                border_width: BUTTON_BORDER_WIDTH,
                border_color: super::WHITE,
            }
        }
    }
}
