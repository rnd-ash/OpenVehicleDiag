use common::raf::{self, Raf};

use self::creader::CaesarPrimitive;

pub mod creader;
pub mod container;
#[derive(Debug)]
pub enum CaesarError {
    FileError(raf::RafError),
    ProcessException(String),
    IOError(std::io::Error)
}

impl From<raf::RafError> for CaesarError {
    fn from(x: raf::RafError) -> Self {
        Self::FileError(x)
    }
}

impl From<std::io::Error> for CaesarError {
    fn from(x: std::io::Error) -> Self {
        Self::IOError(x)
    }
}

pub type Result<T> = std::result::Result<T, CaesarError>;



#[derive(Debug, Copy, Clone, Default)]
pub (crate) struct PoolTuple {
    pub (crate) count: usize,
    pub (crate) offset: usize
}

impl PoolTuple {
    pub fn new_int(reader: &mut Raf, bf: &mut u32) -> Result<Self> {
        Ok(Self{
            count: creader::read_primitive(bf, reader, 0i32)?.to_usize(),
            offset: creader::read_primitive(bf, reader, 0i32)?.to_usize()
        })
    }

    pub fn new_default<T: CaesarPrimitive, X: CaesarPrimitive>(reader: &mut Raf, bf: &mut u32, default_count: T, default_offset: X) -> Result<Self> {
        Ok(Self{
            count: creader::read_primitive(bf, reader, default_count)?.to_usize(),
            offset: creader::read_primitive(bf, reader, default_offset)?.to_usize()
        })
    }
}