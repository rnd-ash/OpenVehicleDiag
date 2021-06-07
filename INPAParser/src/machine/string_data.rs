use std::{cell::RefCell, rc::{Rc, Weak}};



#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringData {
    length: u32,
    data: Vec<u8>
}

impl StringData {
    pub fn new(machine: &super:: Machine, length: u32) -> Self {
        Self {
            length,
            data: Vec::new()
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.length = 0;
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

    pub fn get_data_len(&self) -> usize {
        self.data.len()
    }
}
