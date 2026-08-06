use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    path::Path,
    sync::{Arc, Mutex},
};

use crate::data_mgr::DeviceError;
use crate::utils::file::enforce_private_permissions;

#[derive(Debug, Clone)]
pub struct FileSystemDataManager {
    file: Arc<Mutex<File>>,
}

impl FileSystemDataManager {
    pub fn open_file(path: &str) -> Result<Self, DeviceError> {
        let path = Path::new(path);
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true);

        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }

        let file = options
            .open(path)
            .map_err(|_| DeviceError::DeviceNotFound)?;

        enforce_private_permissions(path).map_err(|_| DeviceError::DeviceNotFound)?;
        Ok(Self {
            file: Arc::new(Mutex::new(file)),
        })
    }
}

impl FileSystemDataManager {
    pub fn init_memory(&mut self, size: usize) -> Result<(), DeviceError> {
        let file = self.file.lock().expect("Failed to lock file");
        file.set_len(size as u64)
            .map_err(|_| DeviceError::InitMemoryError)?;
        Ok(())
    }

    pub fn byte_len(&self) -> Result<u64, DeviceError> {
        let file = self.file.lock().expect("Failed to lock file");
        file.metadata()
            .map(|m| m.len())
            .map_err(|_| DeviceError::ReadError)
    }

    pub fn read_data(&mut self, address: u32, size: usize) -> Result<Vec<u8>, DeviceError> {
        let mut file = self.file.lock().expect("Failed to lock file");

        if file.metadata().map(|m| m.len()).unwrap_or(0) < address as u64 + size as u64 {
            return Err(DeviceError::ReadError);
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
