pub mod elements;
pub mod images;
use crate::themes::elements::{ButtonStyle, DropDown, PBar};
use iced::{Button, Checkbox, Color, Container, Element, PickList, ProgressBar, Radio, Text, button, pick_list};
use std::ops::RangeInclusive;
use std::borrow::Cow;

use self::elements::{ButtonTableStyle, CheckBox, TextInput};

const BUTTON_RADIUS: f32 = 5.0;
const BUTTON_BORDER_WIDTH: f32 = 1.5;
const GREY: Color = Color {
    r: 0x75 as f32 / 255.0,
    g: 0x75 as f32 / 255.0,
    b: 0x75 as f32 / 255.0,
    a: 1.0,
};

const DARK_BG: Color = Color {
    r: 0x12 as f32 / 255.0,
    g: 0x12 as f32 / 255.0,
    b: 0x12 as f32 / 255.0,
    a: 1.0,
};

const WHITE: Color = Color {
    r: 1.0,
    g: 1.0,
    b: 1.0,
    a: 1.0,
};

static mut CURR_THEME: Style = Style::Dark;



static mut DEBUG: bool = false;

pub fn set_debug(state: bool) {
    unsafe { DEBUG = state }
}

pub fn is_debug() -> bool {
    unsafe { DEBUG }
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub enum Style {
    Light,
    Dark,
}

pub fn set_dark_theme() {
    unsafe { CURR_THEME = Style::Dark }
}

pub fn set_light_theme() {
    unsafe { CURR_THEME = Style::Light }
}

pub fn toggle_theme() {
    if *get_theme() == Style::Light {
        set_dark_theme()
    } else {
        set_light_theme()
    }
}

pub(crate) fn get_theme<'a>() -> &'a Style {
    unsafe { &CURR_THEME }
}

pub enum ButtonType {
    Primary,
    Secondary,
    Success,
    Danger,
    Warning,
    Info,
    Light,
    Dark,
}

#[derive(Debug, Copy, Clone)]
pub enum TextType {
    Success,
    Warning,
    Danger,
    Normal,
    Disabled,
}

impl TextType {
    pub(crate) fn get_colour(&self) -> Color {
        match &self {
            TextType::Success => ButtonType::Success.get_colour(),
            TextType::Warning => ButtonType::Warning.get_colour(),
            TextType::Danger => ButtonType::Danger.get_colour(),
            TextType::Disabled => GREY,
            TextType::Normal => {
                if *get_theme() == Style::Light {
                    DARK_BG
                } else {
                    WHITE
                }
            }
        }
    }
}

impl ButtonType {
    pub(crate) fn get_colour(&self) -> Color {
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

pub fn button_coloured<'a, T: Clone>(
    state: &'a mut button::State,
    text: &str,
    btn_type: ButtonType,
) -> Button<'a, T> {
    let color = btn_type.get_colour();
    Button::new(state, Text::new(text))
        .style(ButtonStyle::new(color, false))
        .padding(8)
}

pub fn button_outlined<'a, T: Clone>(
    state: &'a mut button::State,
    text: &str,
    btn_type: ButtonType,
) -> Button<'a, T> {
    let color = btn_type.get_colour();
    Button::new(state, Text::new(text))
        .style(ButtonStyle::new(color, true))
        .padding(8)
}

pub fn button_table<'a, T: Clone>(
    state: &'a mut button::State,
    text: &str,
    btn_type: ButtonType,
    is_selected: bool
) -> Button<'a, T> {
    let color = btn_type.get_colour();
    Button::new(state, Text::new(text))
        .style(ButtonTableStyle::new(color, is_selected))
}

pub fn checkbox<T:Clone, F>(
    is_checked: bool,
    label: &str,
    on_click: F
) -> Checkbox<T> 
where
F: 'static + Fn(bool) -> T {
    Checkbox::new(is_checked, label, on_click)
        .style(CheckBox)
}

pub fn picklist<'a, T, Msg>(
    state: &'a mut pick_list::State<T>,
    options: impl Into<Cow<'a, [T]>>,
    selected: Option<T>,
    on_selected: impl Fn(T) -> Msg + 'static,
) -> PickList<'a, T, Msg>
where
    T: ToString + Eq,
    [T]: ToOwned<Owned = Vec<T>>,
{
    PickList::new(state, options, selected, on_selected)
        .style(DropDown)
        .padding(8)
}

pub fn container<'a, Msg, T>(contents: T) -> Container<'a, Msg>
where
    T: Into<Element<'a, Msg>>,
{
    Container::new(contents).style(elements::Container)
}

pub fn radio_btn<Msg: Clone, V, F>(
    value: V,
    label: impl Into<String>,
    selected: Option<V>,
    f: F,
    btn_t: ButtonType,
) -> Radio<Msg>
where
    V: Eq + Copy,
    F: 'static + Fn(V) -> Msg,
{
    Radio::new(value, label, selected, f).style(elements::RadioBtn::new(btn_t))
}

pub enum TitleSize {
    P1,
    P2,
    P3,
    P4,
}

pub fn title_text(text: &str, size: TitleSize) -> iced::Text {
    Text::new(text).size(match size {
        TitleSize::P1 => 60,
        TitleSize::P2 => 50,
        TitleSize::P3 => 40,
        TitleSize::P4 => 30,
    })
}

pub fn text(text: &str, txt_type: TextType) -> iced::Text {
    Text::new(text).color(txt_type.get_colour())
}

pub fn progress_bar(
    range: RangeInclusive<f32>,
    curr_value: f32,
    c_type: ButtonType,
) -> iced::ProgressBar {
    ProgressBar::new(range, curr_value).style(PBar::new(c_type))
}

pub fn text_input<'a, Msg: Clone, F>(
    state: &'a mut iced::text_input::State,
    placeholder: &str,
    value: &str,
    on_change: F,
) -> iced::text_input::TextInput<'a, Msg>
where
    F: 'static + Fn(String) -> Msg,
{
    iced::text_input::TextInput::new(state, placeholder, value, on_change)
        .style(TextInput)
        .padding(8)
}
