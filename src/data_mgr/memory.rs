use std::sync::{Arc, Mutex};

use crate::data_mgr::DeviceError;

#[derive(Debug, Clone)]
pub struct MemoryDataManager {
    data: Arc<Mutex<Vec<u8>>>,
}

impl MemoryDataManager {
    pub fn new(data: Vec<u8>) -> Self {
        Self {
            data: Arc::new(Mutex::new(data)),
        }
    }
}

impl MemoryDataManager {
    pub fn init_memory(&mut self, size: usize) -> Result<(), DeviceError> {
        let mut data = self.data.lock().expect("Failed to lock data");
        data.resize(size, 0);
        Ok(())
    }

    pub fn read_data(&self, address: u32, size: usize) -> Result<Vec<u8>, DeviceError> {
        let data = self.data.lock().expect("Failed to lock data");

        if address as usize + size > data.len() {
            return Ok(Vec::with_capacity(size));
        }

        Ok(data[address as usize..address as usize + size].to_vec())
    }

    pub fn write_data(&self, address: u32, data: &[u8]) -> Result<(), DeviceError> {
        let mut memory = self.data.lock().expect("Failed to lock memory");

        if address as usize + data.len() > memory.len() {
            memory.resize(address as usize + data.len(), 0);
        }

        memory[address as usize..address as usize + data.len()].copy_from_slice(data);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_data_manager() {
        let memory_mgr = MemoryDataManager::new(vec![0; 1024]);
        let write_data = vec![1, 2, 3, 4, 5];
        memory_mgr.write_data(100, &write_data).unwrap();

        let read_data = memory_mgr.read_data(100, 5).unwrap();
        assert_eq!(read_data, write_data);

        let empty_read = memory_mgr.read_data(2000, 5).unwrap();
        assert!(empty_read.is_empty());
    }
}
