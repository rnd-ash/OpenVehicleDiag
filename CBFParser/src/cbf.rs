use std::io::BufReader;
use std::io::Read;

#[derive(Debug, Clone)]
pub enum CbfError {
    // TODO Add error types
    OutOfBounds,
    UtfParseFail,
}

pub type Result<T> = std::result::Result<T, CbfError>;

pub struct CbfFile {
    data: Vec<u8>,
    pos: usize,
    lim: usize,
}

impl CbfFile {
    pub fn from_buf_reader<R: Read>(mut r: BufReader<R>) -> CbfFile {
        let mut cbf = CbfFile {
            data: Vec::new(),
            pos: 0,
            lim: 0,
        };
        r.read_to_end(&mut cbf.data).unwrap();
        cbf.lim = cbf.data.len();
        cbf
    }

    /// Reads the header text from CBF File
    /// Ignores the padding which results in 0x3FF bytes being read
    pub fn read_header(&mut self) -> Result<String> {
        let buf = self.read_bytes(0x3FF)?;
        return match std::str::from_utf8(&buf) {
            Ok(x) => Ok(String::from(x.trim_matches(char::from(0)))),
            Err(_) => Err(CbfError::UtfParseFail),
        };
    }

    // Skips [count] bytes in the CBF File
    //pub fn skip_bytes(&mut self, count: usize) {
    //let mut tmp: Vec<u8> = vec![0x00; count];
    //self.reader.read_exact(&mut tmp).unwrap();
    //println!("SKIP: {:?}", tmp);
    //}

    pub fn read_until(&mut self, dilemer: u8) -> Result<Vec<u8>> {
        self.read_until_bytes(&mut vec![dilemer])
    }

    pub fn read_until_str(&mut self, dilema: &str) -> Result<Vec<u8>> {
        self.read_until_bytes(dilema.as_bytes())
    }

    pub fn read_until_bytes(&mut self, matcher: &[u8]) -> Result<Vec<u8>> {
        let mut tmpbuf = Vec::with_capacity(matcher.len());

        loop {
            if tmpbuf.len() >= matcher.len()
                && &matcher[..] == &tmpbuf[tmpbuf.len() - matcher.len()..]
            {
                return Ok(tmpbuf);
            }
            tmpbuf.push(*self.read_bytes(1)?.get(0).unwrap());
        }
    }

    /// Peeks at the next byte in the file
    pub fn peek_next_bytes(&mut self, size: usize) -> &[u8] {
        let s = self.pos;
        &self.data[s..s + size]
    }

    /// Reads [count] bytes from the CBF File
    pub fn read_bytes(&mut self, count: usize) -> Result<Vec<u8>> {
        let start = self.pos;
        if start + count > self.lim {
            return Err(CbfError::OutOfBounds);
        }
        self.pos += count;
        return Ok(Vec::from(&self.data[start..start + count]));
    }

    ///Reads a null terminated UTF-8 String from CBF File
    pub fn read_string(&mut self) -> Result<String> {
        let mut pos = self.pos;
        while self.data[pos] == 0x00 {
            pos += 1;
        }
        self.pos = pos;
        let mut bytes = self.read_until(0x00)?;
        bytes.pop();
        return match String::from_utf8(bytes) {
            Ok(s) => Ok(s),
            Err(_) => Err(CbfError::UtfParseFail),
        };
    }
}
