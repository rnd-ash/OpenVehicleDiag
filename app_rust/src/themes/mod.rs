pub mod dark;
pub mod light;

use iced::{button, Color, Button, Text, Length, PickList, pick_list, Container, Application, Element};
use crate::themes::dark::{ButtonStyle, DropDown};
use std::borrow::Cow;

const BUTTON_RADIUS : f32 = 5.0;
const BUTTON_BORDER_WIDTH: f32 = 1.5;
const GREY: Color = Color {
    r: 0x75 as f32 / 255.0,
    g: 0x75 as f32 / 255.0,
    b: 0x75 as f32 / 255.0,
    a: 1.0
};

const DARK_BG: Color = Color {
    r: 0x12 as f32 / 255.0,
    g: 0x12 as f32 / 255.0,
    b: 0x12 as f32 / 255.0,
    a: 1.0
};

const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0
};


static mut CURR_THEME: Style = Style::Dark;

pub enum Style {
    Light,
    Dark
}

pub fn set_dark_theme() {
    unsafe { CURR_THEME = Style::Dark }
}

pub fn set_light_theme() {
    unsafe { CURR_THEME = Style::Light }
}

pub enum ButtonType {
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
    Info,
    Light,
    Dark
}

impl ButtonType {
    pub (crate) fn get_colour(&self) -> Color {
        match &self {
            ButtonType::Primary => Color::from_rgb8(0x0d, 0x6e, 0xfd),
            ButtonType::Secondary => Color::from_rgb8(0x66, 0x10, 0xf2),
            ButtonType::Success => Color::from_rgb8(0x00, 0xb7, 0x4a),
            ButtonType::Danger => Color::from_rgb8(0xf9, 0x31, 0x54),
            ButtonType::Warning => Color::from_rgb8(0xff, 0xa9, 0x00),
            ButtonType::Info => Color::from_rgb8(0x39, 0xc0, 0xed),
            ButtonType::Light => Color::from_rgb8(0xfb, 0xfb, 0xfb),
            ButtonType::Dark => Color::from_rgb8(0x26, 0x26, 0x26),
        }
    }
}

pub fn button_coloured<'a, T: Clone>(state: &'a mut button::State, text: &str, btn_type: ButtonType) -> Button<'a, T> {
    let color = btn_type.get_colour();
    Button::new(state, Text::new(text)).style(
        match unsafe { &CURR_THEME } {
            Style::Light => { unimplemented!() }
            Style::Dark => { ButtonStyle::new(color, false) }
        }
    ).padding(8)
}

pub fn button_outlined<'a, T: Clone>(state: &'a mut button::State, text: &str, btn_type: ButtonType) -> Button<'a, T> {
    let color = btn_type.get_colour();
    Button::new(state, Text::new(text)).style(
        match unsafe { &CURR_THEME } {
            Style::Light => { unimplemented!() }
            Style::Dark => { ButtonStyle::new(color, true) }
        }
    ).padding(8)
}

pub fn picklist<'a, T, Msg>(state: &'a mut pick_list::State<T>, options: impl Into<Cow<'a, [T]>>, selected: Option<T>, on_selected: impl Fn(T) -> Msg + 'static) -> PickList<'a, T, Msg>
where
    T: ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
{
    PickList::new(state, options, selected, on_selected).style(
        match unsafe { &CURR_THEME } {
            Style::Light => { unimplemented!() }
            Style::Dark => { DropDown }
        }
    ).padding(8)
}

pub fn container<'a, Msg, T>(contents: T) -> Container<'a, Msg>
where T : Into<Element<'a, Msg>> {
    Container::new(contents).style(
        match unsafe { &CURR_THEME } {
            Style::Light => { unimplemented!() }
            Style::Dark => { dark::Container }
        }
    )
}

