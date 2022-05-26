use std::collections::HashMap;

#[derive(Default, Debug)]
pub struct MemoryCommander {
    fixed: HashMap<usize, u8>,
}

impl MemoryCommander {
    pub fn set_fixed(&mut self, addr: u16, value: u8) {
        self.fixed.insert(addr as usize, value);
    }

    pub fn clear_fixed(&mut self, addr: u16) {
        self.fixed.remove(&(addr as usize));
    }
}

impl MemoryCommander {
    pub fn update(&mut self, memory: &mut [u8]) {
        for (addr, value) in &self.fixed {
            memory[*addr] = *value;
        }
    }
}
