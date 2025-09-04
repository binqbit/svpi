use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::{Arc, Mutex},
};

use crate::data_mgr::DeviceError;

#[derive(Debug, Clone)]
pub struct FileSystemDataManager {
    file: Arc<Mutex<File>>,
}

impl FileSystemDataManager {
    pub fn open_file(path: &str) -> Result<Self, DeviceError> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(path)
            .map_err(|_| DeviceError::DeviceNotFound)?;
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }
}

impl FileSystemDataManager {
    pub fn init_memory(&mut self, size: usize) -> Result<(), DeviceError> {
        let mut file = self.file.lock().expect("Failed to lock file");
        file.set_len(size as u64)
            .map_err(|_| DeviceError::InitMemoryError)?;
        Ok(())
    }

    pub fn read_data(&mut self, address: u32, size: usize) -> Result<Vec<u8>, DeviceError> {
        let mut file = self.file.lock().expect("Failed to lock file");

        if file.metadata().map(|m| m.len()).unwrap_or(0) < address as u64 + size as u64 {
            return Ok(Vec::with_capacity(size));
        }

        let mut buffer = vec![0; size];
        file.seek(SeekFrom::Start(address.into()))
            .map_err(|_| DeviceError::ReadError)?;
        file.read_exact(&mut buffer)
            .map_err(|_| DeviceError::ReadError)?;
        Ok(buffer)
    }

    pub fn write_data(&mut self, address: u32, data: &[u8]) -> Result<(), DeviceError> {
        let mut file = self.file.lock().expect("Failed to lock file");

        file.seek(SeekFrom::Start(address.into()))
            .map_err(|_| DeviceError::WriteError)?;
        file.write_all(data).map_err(|_| DeviceError::WriteError)?;
        file.sync_data().map_err(|_| DeviceError::WriteError)?;
        Ok(())
    }
}
