#[derive(Debug, Clone, Default)]
pub struct ArgInfo {
    data: Vec<u8>,
    pub string_list: Vec<String>,
}

impl ArgInfo {
    pub fn get_data(&self) -> &[u8] {
        &self.data
    }

    pub fn set_data(&mut self, data: &[u8]) {
        self.data = Vec::from(data);
        self.string_list.clear();
    }
}
