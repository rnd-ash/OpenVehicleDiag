use common::raf::Raf;
use ctf_header::CTFHeader;

use crate::ctf::{STUB_HEADER_SIZE, StubHeader, cff_header::CFFHeader, ctf_header};



#[derive(Debug, Clone, Default)]
pub(crate) struct Container {
    cff_header: CFFHeader,
    ctf_header: CTFHeader,
}

impl Container {
    pub fn new(reader: &mut Raf) -> super::Result<Self> {
        reader.seek(0);

        let header = reader.read_bytes(STUB_HEADER_SIZE)?;
        StubHeader::read_header(&header);

        let cff_header_size = reader.read_i32()? as usize;
        let cff_header_bytes = reader.read_bytes(cff_header_size)?;

        let mut container = Container::default();

        container.cff_header = container.read_cff(reader)?;
        println!("{:#?}", container);
        container.ctf_header = container.read_ctf(reader)?;
        Ok(container)
    }

    fn read_cff(&self, reader: &mut Raf) -> super::Result<CFFHeader> {
        CFFHeader::new(reader)
    }

    fn read_ctf(&self, reader: &mut Raf) -> super::Result<CTFHeader> {
        let offset = self.cff_header.base_addr + self.cff_header.ctf_offset as usize;
        CTFHeader::new(reader, offset, self.cff_header.cff_header_size as usize)
    }
}