use common::raf::Raf;
use ctf_header::CTFHeader;
use ecu::ECU;

use crate::{ctf::{STUB_HEADER_SIZE, StubHeader, cff_header::CFFHeader, ctf_header::{self, CTFLanguage}}, ecu};

#[derive(Debug, Clone, Default)]
pub struct Container {
    pub cff_header: CFFHeader,
    pub ctf_header: CTFHeader,
    pub language: CTFLanguage,
    pub ecus: Vec<ECU>,
}

impl Container {
    pub fn new(reader: &mut Raf) -> super::Result<Self> {
        reader.seek(0);

        let header = reader.read_bytes(STUB_HEADER_SIZE)?;
        StubHeader::read_header(&header);

        let cff_header_size = reader.read_i32()? as usize;
        let _cff_header_bytes = reader.read_bytes(cff_header_size)?;

        let mut container = Container::default();

        container.read_cff(reader)?;
        container.read_ctf(reader)?;
        container.language = container.ctf_header.languages[0].clone();
        Ok(container)
    }

    fn read_cff(&mut self, reader: &mut Raf) -> super::Result<()> {
        self.cff_header = CFFHeader::new(reader)?;
        Ok(())
    }

    fn read_ctf(&mut self, reader: &mut Raf) -> super::Result<()> {
        let offset = self.cff_header.base_addr + self.cff_header.ctf_offset as usize;
        self.ctf_header = CTFHeader::new(reader, offset, self.cff_header.cff_header_size as usize)?;
        Ok(())
    }

    pub fn read_ecus(&mut self, reader: &mut Raf) -> super::Result<()> {
        self.ecus.clear();
        let ecu_table_offset = self.cff_header.ecu_offset as usize + self.cff_header.base_addr;
        reader.seek(ecu_table_offset);
        let offset_to_actual_ecu = reader.read_i32()? as usize;
        let ecu = ECU::new(reader, self.language.clone(), &self.cff_header,ecu_table_offset + offset_to_actual_ecu)?;
        self.ecus.push(ecu);
        Ok(())
    }

    pub fn dump_strings(&self, name: String) {
        if self.ctf_header.languages[0].dump_language_table(name).is_ok() {
            println!("String dump complete. Have a nice day")
        } else {
            eprintln!("String dump failed")
        }
    }

    pub fn load_strings(&mut self, name: String) {
        if self.ctf_header.languages[0].load_language_table(name).is_ok() {
            println!("String loading complete.")
        } else {
            panic!("String load failed")
        }
    }
}