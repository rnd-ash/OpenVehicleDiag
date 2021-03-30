use iced::{image::Handle, Image};

#[cfg(unix)]
pub const TRAY_ICON: &[u8] = include_bytes!("../../img/launcher.png");
#[cfg(windows)]
pub const TRAY_ICON: &[u8] = include_bytes!("..\\..\\img\\launcher.png");

#[cfg(unix)]
pub const LAUNCHER_IMG_LIGHT: &[u8] = include_bytes!("../../img/logo.png");
#[cfg(windows)]
pub const LAUNCHER_IMG_LIGHT: &[u8] = include_bytes!("..\\..\\img\\logo.png");

#[cfg(unix)]
pub const LAUNCHER_IMG_DARK: &[u8] = include_bytes!("../../img/logo_dark.png");
#[cfg(windows)]
pub const LAUNCHER_IMG_DARK: &[u8] = include_bytes!("..\\..\\img\\logo_dark.png");

#[cfg(unix)]
pub const TRAY_ICON_DARK: &[u8] = include_bytes!("../../img/launcher_dark.png");
#[cfg(windows)]
pub const TRAY_ICON_DARK: &[u8] = include_bytes!("..\\..\\img\\launcher_dark.png");


pub fn get_launcher_image() -> iced::Image {
    match super::get_theme() {
        super::Style::Light => pix_to_iced_image(LAUNCHER_IMG_LIGHT),
        super::Style::Dark => pix_to_iced_image(LAUNCHER_IMG_DARK),
    }
}

pub fn pix_to_iced_image(src: &'static [u8]) -> iced::Image {
    let handle = Handle::from_memory(Vec::from(src));
    Image::new(handle)
}
