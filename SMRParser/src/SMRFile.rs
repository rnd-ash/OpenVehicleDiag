use std::{fmt::Write, io::{Cursor, Read}};
use byteorder::{LittleEndian, ByteOrder};
use zip::read::read_zipfile_from_stream;
use std::num::Wrapping;
use nettle::cipher::Blowfish;
use nettle::cipher::Cipher;
use flate2::Decompress;

use super::BlowFishTable;

const SMR_MAGIC : [u8; 16] =  [0x52, 0x90, 0xD4, 0x30, 0x67, 0x14, 0x7E, 0x47, 0x81, 0xF2, 0x3C, 0x4B, 0x73, 0xF0, 0xF7, 0x37];

const DOS_LONG_SIG: &str = "This program cannot be run in DOS mode";

const MZ: &str = "MZ";

const ZIP_SIG: [u8; 4] = [0x50, 0x4B, 0x03, 0x04];
const ZIP_EOCD_SIG: [u8; 4] = [0x50, 0x4B, 0x05, 0x06];

#[derive(Clone)]
pub struct SMRD {
    bytes: Vec<u8>,
    pos: usize,
    lim: usize,
}

pub struct SMRFile {
    reader: SMRD,
    hash_block: Vec<u8>,
    odb_binary: Vec<u8>,
    string_table: Vec<String>,
    meta_info: String,
}

impl SMRD {
    pub fn fromFile<T: Read>(buf: &mut T) -> SMRD {
        let mut t : Vec<u8> = Vec::new();
        buf.read_to_end(&mut t).expect("Fatal error reading SMR-D File");
        let lim = t.len();
        if lim < 0x54 { panic!("SMR File not long enough") }
        SMRD {
            bytes: t,
            pos: 0,
            lim,
        }
    }


    pub fn read_file(&mut self) -> SMRFile {
        // compare first 16 bytes
        if self.read_bytes(0x10) != SMR_MAGIC {
            panic!("Not SMR File (Invalid magic)");
        }

        if self.read_i32() != 0x44 {
            panic!("Incompatible header size! (Not 0x44)")
        }

        let ODBType = self.read_i32();
        let HeaderFileHashBlockOffset = self.read_i32();
        let HeaderClientID = self.read_i32();
        let headerClientXorMaskSize = self.read_i32();

        let headerValue5 = self.read_i32();
        let headerValue6 = self.read_i32();
        let headerValue7 = self.read_i32();

        let headerODBSection1Size = self.read_i32();
        let headerODBSection1Attributes = self.read_i32();
        let headerODBSection2Size = self.read_i32();
        let headerODBSection2Attributes = self.read_i32();
        let headerODBSection3Size = self.read_i32();
        let headerODBSection3Attributes = self.read_i32();


        println!("SMR File. Size: {} bytes", self.lim);
        println!("==> HEADER <==");
        println!("--> Header ODB Type: {:#08x?}", ODBType);
        println!("--> Header file hash block offset: {:#08x?}", HeaderFileHashBlockOffset);
        println!("--> Header client ID: {:#08x?}", HeaderClientID);
        println!("--> Header file hash block mask size: {:#08x?}", headerClientXorMaskSize);
        println!("--> Header value 5: {:#08x?}", headerValue5);
        println!("--> Header value 6: {:#08x?}", headerValue6);
        println!("--> Header value 7: {:#08x?}", headerValue7);
        println!("==> SECTIONS <==");
        println!("--> 1: Size: {:#08x?}", headerODBSection1Size);
        println!("--> 1: Attributes: {:#08x?}", headerODBSection1Attributes);
        println!("--> 2: Size: {:#08x?}", headerODBSection2Size);
        println!("--> 2: Attributes: {:#08x?}", headerODBSection2Attributes);
        println!("--> 3: Size: {:#08x?}", headerODBSection3Size);
        println!("--> 3: Attributes: {:#08x?}", headerODBSection3Attributes);

        self.pos += 3 * 4; // Skip some data 

        let headerMetaInfoSize = self.read_i32();
        let b = self.read_bytes(headerMetaInfoSize as usize);
        let metaInfo = String::from_utf8(b).unwrap();
        println!("==> METADATA <==");
        println!("{}", metaInfo);

        let mask = self.createXORMask(headerClientXorMaskSize);
        let decrypt_key = BlowFishTable::get_blowfish_key_from_cid(HeaderClientID);
        println!("==> Decryption key <==");
        println!("{:02x?}", decrypt_key);


        let mut bf = Blowfish::with_decrypt_key(&decrypt_key).expect("Couldn't parse decrypt key!");
        let hash_block = self.create_data_section(0x20, 0, &mask, Some(&mut bf));
        let unknown_block = self.create_data_section(headerODBSection1Size, headerODBSection1Attributes, &mask, Some(&mut bf));
        let binary_block = self.create_data_section(headerODBSection2Size, headerODBSection2Attributes, &mask, Some(&mut bf));
        let string_block = self.create_data_section(headerODBSection3Size, headerODBSection3Attributes, &mask, Some(&mut bf));

        let strs = self.read_value_table(string_block);
        SMRFile {
            reader: self.clone(),
            hash_block : hash_block,
            odb_binary: binary_block,
            string_table: strs,
            meta_info: metaInfo,
        }
    }



    fn read_value_table(&mut self, index_bytes: Vec<u8>) -> Vec<String> {
        let mut entries: Vec<String> = Vec::new();

        let mut lastPos = 0;
        (0..index_bytes.len()).for_each(|i| {
            if index_bytes[i] == 0 {
                let str_size = i - lastPos as usize;
                let str_data = Vec::from(&index_bytes[lastPos..lastPos+str_size]);
                lastPos = i;
                entries.push(
                    unsafe { String::from_utf8_unchecked(str_data) }
                )
            }
        });
        entries

    }

    fn read_i32(&mut self) -> i32 {
        LittleEndian::read_i32(&self.read_bytes(4))
    }

    fn read_bytes(&mut self, num: usize) -> Vec<u8> {
        let buf = Vec::from(&self.bytes[self.pos..self.pos+num]);
        self.pos += num;
        return buf;
    }

    fn createXORMask(&mut self, maskSize: i32) -> Vec<u8> {
        let mut buf: Vec<u8> = vec![0x00; maskSize as usize];
        let mut state = Wrapping(maskSize as u32);

        (0..maskSize).for_each(|i| {
            state = Wrapping(0x41C64E6D) * state + Wrapping(0x3039);
            buf[i as usize] = ((state.0 >> 16) & 0xFF) as u8;
        });
        return buf;
    }

    fn create_data_section(&mut self, size: i32, attribs: i32, xor_mask: &Vec<u8>, fish: Option<&mut Blowfish>) -> Vec<u8> {
        let curr_pos = self.pos;
        let raw_bytes = self.read_bytes(size as usize);
        return match fish {
            Some(x) => {
                let mut unxor_bytes = self.xor_transform(raw_bytes, curr_pos, xor_mask);
                let mut plain_bytes: Vec<u8> = vec![0x00; unxor_bytes.len()];
                x.decrypt(&mut plain_bytes, &mut unxor_bytes);
                match attribs & 0x100 {
                    d if d > 0 => self.inflate_bytes(plain_bytes),
                    _ => plain_bytes
                }
            }
            None => self.xor_transform(raw_bytes, self.pos, xor_mask)
        }
    }

    fn xor_transform(&mut self, target: Vec<u8>, offset: usize, mask: &Vec<u8>) -> Vec<u8> {
        let mut res = target.clone();
        let end_offset = offset + target.len();
        (offset..end_offset).for_each(|i|{
            let rebase_offset = i - offset;
            res[rebase_offset] = mask[i % mask.len()] ^ target[rebase_offset];
        });
        res
    }

    fn inflate_bytes(&mut self, input: Vec<u8>) -> Vec<u8> {
       
        let expect_size = LittleEndian::read_u32(&input[0..4]) as usize; // 4 bytes
        let zlib_header = LittleEndian::read_u16(&input[4..6]); // 2 bytes

        let remaining_bytes = Vec::from(&input[4..]);
        let mut output_bytes: Vec<u8> = vec![0x00; expect_size];
        println!("Inflating. Source size {}, inflated size: {}, zlib header: {:04x?}", input.len(), expect_size, zlib_header);
        match Decompress::new(true).decompress(&remaining_bytes, &mut output_bytes, flate2::FlushDecompress::Sync) {
            Err(e) => println!("Error decompressing {}", e),
            Ok(_) => println!("Decompressing OK")
        }
        
        output_bytes
    }
}


pub struct WriteFile {
    pub bytes: Vec<u8>,
    pub name: String,
}

impl SMRFile {
    pub fn extract_zips(&mut self) -> Vec<WriteFile> {
        let mut files : Vec<Vec<u8>> = Vec::new();
        let mut fpos = 0;

        while fpos < self.odb_binary.len() {
            let next_zip = match SMRFile::search_bytes(&self.odb_binary, &ZIP_SIG, fpos) {
                -1 => break,
                x => x as usize
            };
            let next_end_zip = match SMRFile::search_bytes(&self.odb_binary, &ZIP_EOCD_SIG, fpos){
                -1 => break,
                x => x as usize
            };

            let x1 = self.odb_binary[(next_end_zip + 0x15) as usize] as u16;
            let x2 = self.odb_binary[(next_end_zip + 0x14)] as u16;

            let zip_comment_length = ((x1 << 8) | x2) as usize;
            let zip_final_length = ((next_end_zip + 0x16 + zip_comment_length) - next_zip) as usize;

            let arr = &self.odb_binary[next_zip..(next_zip + zip_final_length)];
            println!("Zip file found - Size: {}", zip_final_length);
            files.push(Vec::from(arr));
            fpos = next_zip + zip_final_length;
        }
        println!("{} zip file(s) found", files.len());
        // Extract files from SMR-D!

        let mut res = Vec::new();

        for (idx, stream) in files.iter().enumerate() {
            match zip::read::ZipArchive::new(Cursor::new(stream)) {
                Ok(extracted) => {
                    // Manifest, therefore JAR file!
                    if extracted.file_names().into_iter().find(|x| *x == "META-INF/MANIFEST.MF" ).is_some() {
                        println!("JAR File found");
                        // TODO GET JAR name
                        res.push(WriteFile {
                            bytes: stream.clone(),
                            name: format!("{}.jar", idx)
                        })
                    } 
                    // No manifest, assume ZIP
                    else {
                        println!("ZIP File found");
                        res.push(WriteFile {
                            bytes: stream.clone(),
                            name: format!("{}.zip", idx)
                        })
                    }
                },
                Err(e) => {
                    println!("Error: {}", e);
                }
            }
        }
        return res;
    }


    fn search_bytes(src: &Vec<u8>, search: &[u8], offset: usize) -> i32 {
        let lim = src.len() - search.len();
        for i in offset..=lim {
            let mut k = 0;
            while k < search.len() {
                if search[k] != src[i+k] {
                    break;
                }
                k += 1;
            }
            if k == search.len() {
                return i as i32;
            }
        }
        return -1;
    }
}