use iced::{image::Handle, Image};

#[cfg(unix)]
pub const TRAY_ICON: &[u8] = include_bytes!("../../img/launcher.png");
#[cfg(windows)]
pub const TRAY_ICON: &[u8] = include_bytes!("..\\..\\img\\launcher.png");

#[cfg(unix)]
pub const LAUNCHER_IMG: &[u8] = include_bytes!("../../img/logo.png");
#[cfg(windows)]
pub const LAUNCHER_IMG: &[u8] = include_bytes!("..\\..\\img\\logo.png");

#[cfg(unix)]
pub const TRAY_ICON_DARK: &[u8] = include_bytes!("../../img/launcher_dark.png");
#[cfg(windows)]
pub const TRAY_ICON_DARK: &[u8] = include_bytes!("..\\..\\img\\launcher_dark.png");

pub fn pix_to_iced_image(src: &'static [u8]) -> iced::Image {
    let handle = Handle::from_memory(Vec::from(src));
    Image::new(handle)
}
