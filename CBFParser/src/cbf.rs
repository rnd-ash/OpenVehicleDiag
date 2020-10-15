use std::io::BufReader;
use std::io::Read;
use byteorder::LittleEndian;

#[derive(Debug, Clone)]
pub enum CbfError {
    // TODO Add error types
    OutOfBounds,
    UtfParseFail,
}

pub type Result<T> = std::result::Result<T, CbfError>;

pub struct CbfFile {
    bytes: Vec<u8>,
    pos: usize,
    lim: usize
}


impl CbfFile {
    pub fn fromFile<T: Read>(f: &mut BufReader<T>) -> std::io::Result<CbfFile> {
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;
        let len = buf.len();
        Ok(CbfFile {
            bytes: buf,
            pos: 0,
            lim: len
        })
    }

    /// Attempts to reads [numBytes] bytes from the file
    fn readFile(&mut self, numBytes: usize) -> Option<Vec<u8>> {
        if self.pos + numBytes >= self.lim {
            eprintln!("Out of bounds reading! Pos: {} - {} ({} bytes). Max: {}", self.pos, self.pos+numBytes, numBytes, self.lim);
            return None
        }
        let buf = Vec::from(&self.bytes[self.pos..self.pos+numBytes]);
        self.pos += numBytes;
        Some(buf)
    }

    /// Reads [num_bytes] from the file, starting at position [distance_to_move]
    pub fn readData(&mut self, distance_to_move: usize, num_bytes: usize) -> Option<Vec<u8>> {
        self.seekFile( distance_to_move, false)?;
        self.readFile(num_bytes)
    }

    /// Seeks to a virtual address within the data
    ///
    /// # Params
    /// * move_current - True - Offset will be added to the current file offset
    ///                  False - Offset will be from start of the file
    /// * move_amount - offset in bytes to seek
    fn seekFile(&mut self, move_amount: usize, move_current: bool) -> Option<()> {
        let target_pos = match move_current {
            false => move_amount,
            true => move_amount + self.pos
        };
        if target_pos >= self.lim { 
            eprintln!("Seek out of range of file!");
            return None 
        }
        self.pos = target_pos;
        return Some(())
    }

    pub fn getOffset(&mut self, a1: i16, a2: i16, a3: i32) -> i16 {
        let mut v8: [i16; 6] = [0,1,2,3,4,5];
        let mut v3 = 0;
        let mut v4 = 1;
        let mut v7 = v8[0];
        let mut v5 = 1;


        if a1 >= 1 {
            loop {
                if v4 & a3+v3 != 0 {
                    if a1 != v5 {
                        v7 += v8[v5 as usize];
                    }
                } else if a1 == v5 && !(v4 & a3+v3 == 1) {
                    v7 = 0;
                }
                if v4 == 128 {
                    v3 += 1;
                    v4 = 1;
                } else {
                    v4 *= 2;
                }
                v5 += 1;
                if a1 < v5 { break }
            }
        }
        return v7;
    }
}
