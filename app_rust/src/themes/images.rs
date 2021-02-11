use iced::{Image, image::Handle};



#[cfg(unix)]
pub const TRAY_ICON: &[u8] = include_bytes!("../../img/launcher.png");
#[cfg(windows)]
pub const TRAY_ICON: &[u8] = include_bytes!("..\\..\\img\\launcher.png");


#[cfg(unix)]
pub const LAUNCHER_IMG: &[u8] = include_bytes!("../../img/logo.png");
#[cfg(windows)]
pub const LAUNCHER_IMG: &[u8] = include_bytes!("..\\..\\img\\logo.png");


pub fn pix_to_iced_image(src: &'static [u8]) -> iced::Image {
    let handle = Handle::from_memory(Vec::from(src));
    Image::new(handle)
}