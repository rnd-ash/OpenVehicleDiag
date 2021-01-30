use std::todo;

use common::raf::Raf;

use crate::{caesar::CaesarError, ctf::ctf_header::CTFLanguage};



#[derive(Debug, Clone, Default)]
pub struct Preparation {

}

impl Preparation {
    pub fn new(reader: &mut Raf, base_addr: usize, pool_idx: usize, lang: &CTFLanguage) -> std::result::Result<Self, CaesarError> {
        todo!()
    }
}