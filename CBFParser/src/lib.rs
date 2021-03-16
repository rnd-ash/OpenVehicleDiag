use std::{fs::File, io::Read};

pub mod caesar;
pub mod ctf;
pub mod ecu;
pub mod diag;

pub fn read_cbf_complete(src: &mut File) -> caesar::Result<caesar::container::Container> {
    let mut buffer = vec![0; src.metadata().unwrap().len() as usize];
    src.read_exact(&mut buffer)?;
    let mut br = common::raf::Raf::from_bytes(&buffer, common::raf::RafByteOrder::LE);
    let (mut container, raf) = caesar::container::Container::new(&mut br)?;
    container.read_ecus(raf)?;
    Ok(container)
}