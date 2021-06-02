

#[derive(Debug, Clone)]
pub struct StringData<'a> {
    machine: &'a super::Machine,
    length: u32,
    data: Vec<u8>
}

impl<'a> StringData<'a> {
    pub fn new(machine: &'a super:: Machine, length: u32) -> Self {
        Self {
            machine,
            length,
            data: Vec::new()
        }
    }

    pub fn new_array_length(&mut self, length: u32) {
        if length as usize > self.data.len() {
            self.data = vec![0; length as usize]
        }
    }

    pub fn get_data(&self, complete: bool) -> Vec<u8> {
        if complete {
            self.data.clone()
        } else {
            Vec::from(&self.data[0..self.length as usize])
        }
    }
}
