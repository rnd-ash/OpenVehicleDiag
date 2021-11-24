use std::{convert::TryInto, io::Read};

/// Random Access file
///
/// Represents a stream of bytes
/// that can be read in order
/// or read data at specific offsets
#[derive(Debug, Clone, Default)]
pub struct Raf {
    /// Data in bytes
    data: Vec<u8>,
    /// Max size of buffer
    size: usize,
    /// Current pos in buffer
    pub pos: usize,
    /// Byte order
    bo: RafByteOrder,
}

pub type Result<T> = std::result::Result<T, RafError>;

/// Errors that can be returned during reading of data
#[derive(Debug)]
pub enum RafError {
    /// End index requested exceeds the size of the data stored
    BufferOverflow,
    /// Start index of requested data is more than the max data stored
    StartOutOfRange,
    /// String parse failed. Due to invalid UTF8 Characters
    StrParseError,
}

/// Byte order representation struct
#[derive(Debug, Copy, Clone)]
pub enum RafByteOrder {
    /// Big endian
    BE,
    /// Little endian
    LE,
}

impl Default for RafByteOrder {
    fn default() -> Self {
        Self::LE
    }
}

impl Raf {
    /// Creates a [Raf] struct from anything implimenting the [Read]
    /// trait
    ///
    /// # Params
    /// * reader - implimentor of the [Read] trait to be read into a [Raf]
    /// * bo - Byte order of the source data
    ///
    /// # Returns
    /// * Result, Raf is returned if read was successful, else Err(x) is returned
    pub fn from_read<R: Read>(reader: &mut R, bo: RafByteOrder) -> std::io::Result<Self> {
        let mut data: Vec<u8> = Vec::new();
        reader.read_to_end(&mut data).map(|size| Raf {
            data,
            size,
            pos: 0,
            bo,
        })
    }

    /// Creates a [Raf] struct from a Vector of bytes
    /// 
    /// # Params
    /// * data - Original source data - Will be cloned
    /// * bo - Byte order of the source data
    pub fn from_bytes(data: &[u8], bo: RafByteOrder) -> Self {
        Raf {
            data: Vec::from(data),
            size: data.len(),
            pos: 0,
            bo,
        }
    }


    pub fn read_bytes(&mut self, num_bytes: usize) -> Result<Vec<u8>> {
        if self.pos+num_bytes-1 > self.size {
            return Err(RafError::BufferOverflow);
        }
        let res = Vec::from(&self.data[self.pos..self.pos + num_bytes]);
        self.pos += num_bytes;
        Ok(res)
    }

    pub fn read_bytes_as_generic<const SIZE: usize>(&mut self) -> Result<[u8; SIZE]> {
        if self.pos+SIZE-1 > self.size {
            return Err(RafError::BufferOverflow);
        }
        self.pos += SIZE;
        Ok(self.data[self.pos-SIZE..self.pos].try_into().unwrap())

    }

    /// Seeks to location within the data stored
    pub fn seek(&mut self, pos: usize) {
        self.pos = pos;
    }

    pub fn adv(&mut self, pos: usize) -> Result<()> {
        match pos {
            x if self.pos + x > self.size => Err(RafError::StartOutOfRange),
            _ => {
                self.pos += pos;
                Ok(())
            },
        }
    }

    /// Seeks to a position within the file prior to running [func].
    ///
    /// The position in the buffer will be subsequently set to the location
    /// where reading completed.
    /// 
    /// # Example
    /// ```
    /// let data: Vec<u8> = (0x00..0xFF).collect();
    /// let mut reader: Raf = Raf::from_bytes(&data, RafByteOrder::BE);
    /// reader.seek_read(2, Raf::read_i32); // Seeks to position 2 and reads i32
    /// ```
    ///
    /// # Params
    /// * pos - Position in file to start reading from
    /// * func - Function to run to read data
    pub fn seek_read<R>(&mut self, pos: usize, func: fn(&mut Self) -> Result<R>) -> Result<R> {
        self.seek(pos);
        func(self)
    }

    #[inline]
    fn read_primitive<T, const SIZE: usize>(
        &mut self,
        func_le: fn([u8; SIZE]) -> T,
        func_be: fn([u8; SIZE]) -> T,
    ) -> Result<T> {
        match self.bo {
            RafByteOrder::BE => self.read_bytes_as_generic::<SIZE>().map(|r| func_be(r)),
            RafByteOrder::LE => self.read_bytes_as_generic::<SIZE>().map(|r| func_le(r)),
        }
    }

    /// Reads a C String (Ends in 0x00)
    pub fn read_cstr_bytes(&mut self) -> Result<Vec<u8>> {
        let mut bytes: Vec<u8> = Vec::new();
        loop {
            let next_byte = self.read_u8()?;
            if next_byte == 0 {
                return Ok(bytes)
            } else {
                bytes.push(next_byte);
            }
        }
    }

    /// Reads f32 from data at current position in buffer
    pub fn read_f32(&mut self) -> Result<f32> {
        self.read_primitive::<f32, 4>(f32::from_le_bytes, f32::from_be_bytes)
    }

    /// Reads u64 from data at current position in buffer
    pub fn read_u64(&mut self) -> Result<u64> {
        self.read_primitive::<u64, 8>(u64::from_le_bytes, u64::from_be_bytes)
    }

    /// Reads i64 from data at current position in buffer
    pub fn read_i64(&mut self) -> Result<i64> {
        self.read_primitive::<i64, 8>(i64::from_le_bytes, i64::from_be_bytes)
    }

    /// Reads u32 from data at current position in buffer
    pub fn read_u32(&mut self) -> Result<u32> {
        self.read_primitive::<u32, 4>(u32::from_le_bytes, u32::from_be_bytes)
    }

    /// Reads i32 from data at current position in buffer
    pub fn read_i32(&mut self) -> Result<i32> {
        self.read_primitive::<i32, 4>(i32::from_le_bytes, i32::from_be_bytes)
    }

    /// Reads u16 from data at current position in buffer
    pub fn read_u16(&mut self) -> Result<u16> {
        self.read_primitive::<u16, 2>(u16::from_le_bytes, u16::from_be_bytes)
    }

    /// Reads i16 from data at current position in buffer
    pub fn read_i16(&mut self) -> Result<i16> {
        self.read_primitive::<i16, 2>(i16::from_le_bytes, i16::from_be_bytes)
    }

    /// Reads a single byte from data at current position in buffer
    pub fn read_u8(&mut self) -> Result<u8> {
        self.read_byte()
    }

    /// Reads a single byte from data at current position in buffer
    pub fn read_i8(&mut self) -> Result<i8> {
        self.read_byte().map(|x| x as i8)
    }

    pub fn read_byte(&mut self) -> Result<u8> {
        if self.pos > self.size {
            return Err(RafError::StartOutOfRange);
        }
        let res = self.data[self.pos];
        self.pos += 1;
        Ok(res)
    }
}
