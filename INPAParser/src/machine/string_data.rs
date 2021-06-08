use std::{
    cell::RefCell,
    rc::{Rc, Weak},
};

use super::Machine;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct StringData {
    length: u32,
    data: Vec<u8>,
}

impl StringData {
    pub fn new(length: u32) -> Self {
        Self {
            length,
            data: Vec::new(),
        }
    }

    pub fn clear(&mut self) {
        self.data.clear();
        self.length = 0;
    }

    pub fn set_data(&mut self, m: &mut Machine, v: &[u8], keep_length: bool) {
        if v.len() > self.data.len() {
            // TODO Set error BIP_0001
            return;
        }
        self.data[0..v.len()].copy_from_slice(v);
        if !keep_length {
            self.length = v.len() as u32
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

    pub fn get_data_len(&self) -> usize {
        self.data.len()
    }
}
