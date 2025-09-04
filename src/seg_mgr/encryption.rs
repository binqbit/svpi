use crate::{
    seg_mgr::{Data, DataError, DataType},
    utils::crypto::{decrypt, encrypt},
};

impl Data {
    pub fn encrypt(&self, password: &[u8]) -> Result<Vec<u8>, DataError> {
        let data = self.to_bytes()?;
        encrypt(&data, password).ok_or(DataError::EncryptionError)
    }
}

impl DataType {
    pub fn decrypt(&self, data: &[u8], password: &[u8]) -> Result<Data, DataError> {
        let data = decrypt(data, password).ok_or(DataError::DecryptionError)?;
        let data = self.from_bytes(&data)?;
        Ok(data)
    }
}
