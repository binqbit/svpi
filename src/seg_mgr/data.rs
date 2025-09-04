use base58::{FromBase58, ToBase58};
use base64::Engine;
use borsh_derive::{BorshDeserialize, BorshSerialize};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::str::FromStr;
use thiserror::Error;

pub const DATA_NAME_SIZE: usize = 32;
pub const DATA_FINGERPRINT_SIZE: usize = 4;

#[derive(
    Debug, Clone, Copy, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize,
)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    Binary,
    Plain,
    Hex,
    Base58,
    Base64,
    EncryptionKey,
}

#[derive(Debug, Clone, PartialEq, BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Data {
    Binary(Vec<u8>),
    Plain(String),
    Hex(String),
    Base58(String),
    Base64(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, BorshSerialize, BorshDeserialize)]
pub struct DataFingerprint {
    pub fingerprint: [u8; DATA_FINGERPRINT_SIZE],
    pub probe: u8,
}

#[derive(Debug, Clone, Copy, BorshSerialize, BorshDeserialize)]
pub struct DataInfo {
    pub name: [u8; DATA_NAME_SIZE],
    pub address: u32,
    pub size: u32,
    pub data_type: DataType,
    pub password_fingerprint: Option<[u8; DATA_FINGERPRINT_SIZE]>,
    pub fingerprint: DataFingerprint,
}

pub struct FormattedData {
    pub name: String,
    pub data: Data,
    pub data_type: DataType,
    pub password_fingerprint: Option<[u8; DATA_FINGERPRINT_SIZE]>,
}

#[derive(Debug, Error)]
pub enum DataError {
    #[error("UTF-8 decoding error")]
    Utf8DecodeError(std::string::FromUtf8Error),
    #[error("Hex decoding error")]
    HexDecodeError(hex::FromHexError),
    #[error("Base58 decoding error")]
    Base58DecodeError(base58::FromBase58Error),
    #[error("Base64 decoding error")]
    Base64DecodeError(base64::DecodeError),
    #[error("Unpacking error")]
    UnpackError,
    #[error("Error encrypting data")]
    EncryptionError,
    #[error("Error decrypting data")]
    DecryptionError,
    #[error("Invalid data")]
    InvalidData,
}

impl Data {
    pub fn to_bytes(&self) -> Result<Vec<u8>, DataError> {
        match self {
            Data::Binary(data) => Ok(data.clone()),
            Data::Plain(data) => Ok(data.as_bytes().to_vec()),
            Data::Hex(data) => hex::decode(data).map_err(DataError::HexDecodeError),
            Data::Base58(data) => data.from_base58().map_err(DataError::Base58DecodeError),
            Data::Base64(data) => base64::engine::general_purpose::STANDARD
                .decode(&data)
                .map_err(DataError::Base64DecodeError),
        }
    }

    pub fn get_type(&self) -> DataType {
        match self {
            Data::Binary(_) => DataType::Binary,
            Data::Plain(_) => DataType::Plain,
            Data::Hex(_) => DataType::Hex,
            Data::Base58(_) => DataType::Base58,
            Data::Base64(_) => DataType::Base64,
        }
    }

    pub fn convert_to_type(&self, data_type: DataType) -> Result<Data, DataError> {
        let data = self.to_bytes()?;
        data_type.from_bytes(&data)
    }

    pub fn to_string_typed(&self, data_type: DataType) -> Result<String, DataError> {
        let data = self.convert_to_type(data_type)?;
        match data {
            Data::Binary(data) => Ok(hex::encode(data)),
            Data::Plain(text) => Ok(text),
            Data::Hex(hex_str) => Ok(hex_str),
            Data::Base58(base58_str) => Ok(base58_str),
            Data::Base64(base64_str) => Ok(base64_str),
        }
    }

    pub fn to_string(&self) -> Result<String, DataError> {
        self.to_string_typed(self.get_type())
    }

    pub fn from_str_infer(str: &str) -> Data {
        if Data::Hex(str.to_string()).to_bytes().is_ok() {
            Data::Hex(str.to_string())
        } else if Data::Base58(str.to_string()).to_bytes().is_ok() {
            Data::Base58(str.to_string())
        } else if Data::Base64(str.to_string()).to_bytes().is_ok() {
            Data::Base64(str.to_string())
        } else {
            Data::Plain(str.to_string())
        }
    }
}

impl DataType {
    pub fn from_bytes(&self, data: &[u8]) -> Result<Data, DataError> {
        match self {
            DataType::Binary => Ok(Data::Binary(data.to_vec())),
            DataType::Plain => String::from_utf8(data.to_vec())
                .map(Data::Plain)
                .map_err(DataError::Utf8DecodeError),
            DataType::Hex => Ok(Data::Hex(hex::encode(&data))),
            DataType::Base58 => Ok(Data::Base58(data.to_base58())),
            DataType::Base64 => Ok(Data::Base64(
                base64::engine::general_purpose::STANDARD.encode(&data),
            )),
            DataType::EncryptionKey => Ok(Data::Binary(data.to_vec())),
        }
    }

    pub fn from_string(&self, data: &str) -> Result<Data, DataError> {
        match self {
            DataType::Binary => Ok(Data::Binary(
                hex::decode(data).map_err(DataError::HexDecodeError)?,
            )),
            DataType::Plain => Ok(Data::Plain(data.to_string())),
            DataType::Hex => Ok(Data::Hex(data.to_string())),
            DataType::Base58 => Ok(Data::Base58(data.to_string())),
            DataType::Base64 => Ok(Data::Base64(data.to_string())),
            DataType::EncryptionKey => Ok(Data::Binary(
                hex::decode(data).map_err(DataError::HexDecodeError)?,
            )),
        }
    }
}

impl ToString for DataType {
    fn to_string(&self) -> String {
        format!("{:?}", self).to_lowercase()
    }
}

impl FromStr for DataType {
    type Err = DataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "binary" => Ok(DataType::Binary),
            "plain" => Ok(DataType::Plain),
            "hex" => Ok(DataType::Hex),
            "base58" => Ok(DataType::Base58),
            "base64" => Ok(DataType::Base64),
            _ => Err(DataError::InvalidData),
        }
    }
}

impl DataInfo {
    pub fn new(
        name_str: &str,
        address: u32,
        data: &[u8],
        data_type: DataType,
        password_fingerprint: Option<[u8; DATA_FINGERPRINT_SIZE]>,
        fingerprints: &[DataFingerprint],
    ) -> Self {
        let fingerprint = DataFingerprint::find_unique(data, fingerprints);

        let mut name = [0u8; DATA_NAME_SIZE];
        let name_len = name_str.len().min(DATA_NAME_SIZE);
        name[..name_len].copy_from_slice(name_str.as_bytes());

        Self {
            name,
            address,
            size: data.len() as u32,
            data_type,
            password_fingerprint,
            fingerprint,
        }
    }
}

impl DataFingerprint {
    pub fn from(fingerprint: [u8; DATA_FINGERPRINT_SIZE]) -> Self {
        Self {
            fingerprint,
            probe: 0,
        }
    }

    pub fn get_fingerprint(data: &[u8]) -> [u8; DATA_FINGERPRINT_SIZE] {
        let mut hasher = sha2::Sha256::new();
        hasher.update(data);
        let hash = hasher.finalize();

        let fingerprint: [u8; DATA_FINGERPRINT_SIZE] = hash[..DATA_FINGERPRINT_SIZE]
            .try_into()
            .expect("Hash is not long enough for fingerprint");

        fingerprint
    }

    pub fn find_unique(data: &[u8], fingerprints: &[DataFingerprint]) -> Self {
        let fingerprint = Self::get_fingerprint(data);

        let mut fingerprint = DataFingerprint {
            fingerprint,
            probe: 0,
        };

        while fingerprints.iter().any(|f| f == &fingerprint) {
            fingerprint.probe += 1;
            if fingerprint.probe == u8::MAX {
                panic!("Too many collisions, cannot find unique fingerprint");
            }
        }

        fingerprint
    }
}

impl ToString for DataFingerprint {
    fn to_string(&self) -> String {
        hex::encode(&self.fingerprint)
    }
}

impl FromStr for DataFingerprint {
    type Err = DataError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let fingerprint = DataType::Hex.from_string(s)?.to_bytes()?;
        Ok(DataFingerprint {
            fingerprint: fingerprint.try_into().map_err(|_| DataError::InvalidData)?,
            probe: 0,
        })
    }
}

impl FormattedData {
    pub fn new(
        name: String,
        data: Data,
        data_type: DataType,
        password_fingerprint: Option<[u8; DATA_FINGERPRINT_SIZE]>,
    ) -> Self {
        Self {
            name,
            data,
            data_type,
            password_fingerprint,
        }
    }

    pub fn from(
        name: String,
        data: String,
        data_type: String,
        password_fingerprint: Option<String>,
    ) -> Result<Self, DataError> {
        if name.len() > DATA_NAME_SIZE {
            return Err(DataError::InvalidData);
        }

        let data_type = DataType::from_str(&data_type)?;
        let data = data_type
            .from_string(&data)
            .map_err(|_| DataError::InvalidData)?;
        let password_fingerprint = if let Some(pf) = password_fingerprint {
            Some(DataFingerprint::from_str(&pf)?.fingerprint)
        } else {
            None
        };

        Ok(Self {
            name,
            data,
            data_type,
            password_fingerprint,
        })
    }

    pub fn encode(&self) -> Result<String, DataError> {
        let name = self.name.clone();
        let data = self.data.to_string()?;
        let data_type = self.data_type.to_string();
        let password_fingerprint = self
            .password_fingerprint
            .map_or_else(String::new, |pf| DataFingerprint::from(pf).to_string());

        Ok(format!(
            "{} = data:application/vnd.binqbit.svpi;{}{},{}",
            name, password_fingerprint, data_type, data
        ))
    }

    pub fn decode(data: &str) -> Result<Self, DataError> {
        let (name, data) = data.split_once("=").ok_or(DataError::InvalidData)?;
        let name = name.trim().to_string();
        let parts = data.split(";").collect::<Vec<&str>>();

        let (password_fingerprint, data_type, data) = if parts.len() == 3 {
            let password_fingerprint = parts[1].trim_start_matches("fp=");
            let (data_type, data) = parts[2].split_once(",").ok_or(DataError::InvalidData)?;
            (
                Some(password_fingerprint.to_string()),
                data_type.to_string(),
                data.to_string(),
            )
        } else if parts.len() == 2 {
            let (data_type, data) = parts[1].split_once(",").ok_or(DataError::InvalidData)?;
            (None, data_type.to_string(), data.to_string())
        } else {
            return Err(DataError::InvalidData);
        };

        FormattedData::from(name, data, data_type, password_fingerprint)
    }
}

// #[cfg(test)]
// mod tests {
//     use super::*;

//     const PASSWORD: &str = "secret";

//     #[test]
//     fn test_data_to_bytes_roundtrip() {
//         let txt = "hello";

//         // Plain
//         let plain = Data::Plain(txt.to_string());
//         assert_eq!(plain.to_bytes(None).unwrap(), txt.as_bytes());

//         // Hex
//         let hex = Data::Hex(hex::encode(txt));
//         assert_eq!(hex.to_bytes(None).unwrap(), txt.as_bytes());

//         // Base64
//         let b64 = Data::Base64(base64::engine::general_purpose::STANDARD.encode(txt));
//         assert_eq!(b64.to_bytes(None).unwrap(), txt.as_bytes());

//         // Base58
//         let b58 = Data::Base58(txt.as_bytes().to_base58());
//         assert_eq!(b58.to_bytes(None).unwrap(), txt.as_bytes());

//         // Binary
//         let bin = Data::Binary(txt.as_bytes().to_vec());
//         assert_eq!(bin.to_bytes(None).unwrap(), txt.as_bytes());
//     }

//     #[test]
//     fn test_data_encryption_roundtrip() {
//         let bin = Data::Binary(b"hello".to_vec());
//         let encrypted = bin.to_bytes(Some(PASSWORD)).unwrap();
//         assert_ne!(encrypted, b"hello".to_vec());

//         let decrypted = DataType::Binary
//             .from_bytes(&encrypted, true, Some(PASSWORD))
//             .unwrap();
//         assert_eq!(decrypted, bin);

//         // wrong password should return encrypted data
//         let wrong = DataType::Binary
//             .from_bytes(&encrypted, true, Some("wrong"))
//             .unwrap();
//         assert!(matches!(wrong, Data::Encrypted(_)));
//     }

//     #[test]
//     fn test_convert_between_types() {
//         let plain = Data::Plain("hi".to_string());
//         let hex = plain.convert_to_type(DataType::Hex).unwrap();
//         assert_eq!(hex, Data::Hex("6869".to_string()));

//         let base64 = hex.convert_to_type(DataType::Base64).unwrap();
//         assert_eq!(base64, Data::Base64("aGk=".to_string()));

//         let bin = base64.convert_to_type(DataType::Binary).unwrap();
//         assert_eq!(bin, Data::Binary(b"hi".to_vec()));
//     }

//     #[test]
//     fn test_to_string_methods() {
//         let bin = Data::Binary(b"hi".to_vec());
//         assert_eq!(bin.to_string().unwrap(), "6869");

//         let plain = Data::Plain("hello".to_string());
//         assert_eq!(plain.to_string().unwrap(), "hello");

//         let base58 = Data::Base58(b"hi".to_base58());
//         assert_eq!(base58.to_string().unwrap(), "8wr");
//     }

//     #[test]
//     fn test_detect_type() {
//         assert!(matches!(Data::detect_type("6869"), Data::Hex(_)));
//         assert!(matches!(Data::detect_type("aGk="), Data::Base64(_)));
//         assert!(matches!(Data::detect_type("8wr"), Data::Base58(_)));
//         assert!(matches!(Data::detect_type("hello"), Data::Plain(_)));
//     }

//     #[test]
//     fn test_formatted_data_encode_decode() {
//         let fd = FormattedData::new(
//             "name".to_string(),
//             Data::Plain("value".to_string()),
//             DataType::Plain,
//         );
//         let encoded = fd.encode().unwrap();
//         assert_eq!(encoded, "Plain/name: value");

//         let decoded = FormattedData::decode(&encoded.to_lowercase(), None).unwrap();
//         assert_eq!(decoded.name, fd.name);
//         assert_eq!(decoded.data, fd.data);
//         assert_eq!(decoded.data_type, fd.data_type);

//         // Encrypted variant
//         let encrypted = Data::Binary(b"secret".to_vec())
//             .to_bytes(Some(PASSWORD))
//             .unwrap();
//         let encoded = FormattedData::new(
//             "enc".to_string(),
//             Data::Encrypted(encrypted.clone()),
//             DataType::Binary,
//         )
//         .encode()
//         .unwrap();

//         let decoded = FormattedData::decode(&encoded.to_lowercase(), Some(PASSWORD)).unwrap();
//         assert_eq!(decoded.name, "enc");
//         assert_eq!(decoded.data_type, DataType::Binary);
//         assert_eq!(decoded.data, Data::Binary(b"secret".to_vec()));
//     }
// }
