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
        data_type: Option<String>,
        password_fingerprint: Option<String>,
    ) -> Result<Self, DataError> {
        if name.len() > DATA_NAME_SIZE {
            return Err(DataError::InvalidData);
        }

        let (data_type, data) = if let Some(data_type) = data_type {
            let data_type = DataType::from_str(&data_type)?;
            let data = if password_fingerprint.is_some() {
                DataType::Hex
                    .from_string(&data)
                    .map_err(|_| DataError::InvalidData)?
            } else {
                data_type
                    .from_string(&data)
                    .map_err(|_| DataError::InvalidData)?
            };
            (data_type, data)
        } else {
            let data = Data::from_str_infer(&data);
            let data_type = data.get_type();
            (data_type, data)
        };

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

        if let Some(pf) = self.password_fingerprint {
            Ok(format!(
                "{} = data:application/vnd.binqbit.svpi;fp={};{},{}",
                name,
                DataFingerprint::from(pf).to_string(),
                data_type,
                data
            ))
        } else {
            Ok(format!("{} = {}", name, data))
        }
    }

    pub fn decode(data: &str) -> Result<Self, DataError> {
        let (name, data) = data.split_once("=").ok_or(DataError::InvalidData)?;
        let name = name.trim().to_string();
        let parts = data.split(";").collect::<Vec<&str>>();

        let (password_fingerprint, data_type, data) = if parts.len() == 1 {
            (None, None, parts[0].trim().to_string())
        } else {
            let password_fingerprint = parts[1].trim_start_matches("fp=");
            let (data_type, data) = parts[2].split_once(",").ok_or(DataError::InvalidData)?;
            (
                Some(password_fingerprint.to_string()),
                Some(data_type.to_string()),
                data.to_string(),
            )
        };

        FormattedData::from(name, data, data_type, password_fingerprint)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn data_to_bytes_for_each_variant() {
        let txt = "hello";

        let plain = Data::Plain(txt.to_string());
        assert_eq!(plain.to_bytes().unwrap(), txt.as_bytes());

        let hex = Data::Hex(hex::encode(txt));
        assert_eq!(hex.to_bytes().unwrap(), txt.as_bytes());

        let base64 = Data::Base64(base64::engine::general_purpose::STANDARD.encode(txt));
        assert_eq!(base64.to_bytes().unwrap(), txt.as_bytes());

        let base58 = Data::Base58(txt.as_bytes().to_base58());
        assert_eq!(base58.to_bytes().unwrap(), txt.as_bytes());

        let binary = Data::Binary(txt.as_bytes().to_vec());
        assert_eq!(binary.to_bytes().unwrap(), txt.as_bytes());
    }

    #[test]
    fn datatype_from_bytes_for_each_variant() {
        let bytes = b"hi";

        assert_eq!(
            DataType::Binary.from_bytes(bytes).unwrap(),
            Data::Binary(bytes.to_vec())
        );
        assert_eq!(
            DataType::Plain.from_bytes(bytes).unwrap(),
            Data::Plain("hi".to_string())
        );
        assert_eq!(
            DataType::Hex.from_bytes(bytes).unwrap(),
            Data::Hex(hex::encode(bytes))
        );
        assert_eq!(
            DataType::Base58.from_bytes(bytes).unwrap(),
            Data::Base58(bytes.to_base58())
        );
        assert_eq!(
            DataType::Base64.from_bytes(bytes).unwrap(),
            Data::Base64(base64::engine::general_purpose::STANDARD.encode(bytes))
        );
        assert_eq!(
            DataType::EncryptionKey.from_bytes(bytes).unwrap(),
            Data::Binary(bytes.to_vec())
        );
    }

    #[test]
    fn convert_between_types() {
        let plain = Data::Plain("hi".to_string());
        assert_eq!(
            plain.convert_to_type(DataType::Hex).unwrap(),
            Data::Hex("6869".to_string())
        );
        assert_eq!(
            plain.convert_to_type(DataType::Base64).unwrap(),
            Data::Base64("aGk=".to_string())
        );
        assert_eq!(
            plain.convert_to_type(DataType::Base58).unwrap(),
            Data::Base58(b"hi".to_base58())
        );
        assert_eq!(
            plain.convert_to_type(DataType::Binary).unwrap(),
            Data::Binary(b"hi".to_vec())
        );
    }

    #[test]
    fn to_string_variants() {
        let bin = Data::Binary(b"hi".to_vec());
        assert_eq!(bin.to_string().unwrap(), "6869");

        let plain = Data::Plain("hello".to_string());
        assert_eq!(plain.to_string().unwrap(), "hello");

        let base58 = Data::Base58(b"hi".to_base58());
        assert_eq!(base58.to_string().unwrap(), b"hi".to_base58());
    }

    #[test]
    fn from_str_infer_detects_type() {
        assert_eq!(Data::from_str_infer("6869"), Data::Hex("6869".to_string()));
        assert_eq!(
            Data::from_str_infer("aGk="),
            Data::Base64("aGk=".to_string())
        );
        assert_eq!(Data::from_str_infer("8wr"), Data::Base58("8wr".to_string()));
        assert_eq!(
            Data::from_str_infer("hello"),
            Data::Plain("hello".to_string())
        );
    }

    #[test]
    fn datatype_from_string_and_from_str() {
        assert_eq!(
            DataType::Binary.from_string("6869").unwrap(),
            Data::Binary(b"hi".to_vec())
        );
        assert_eq!(
            DataType::Plain.from_string("hi").unwrap(),
            Data::Plain("hi".to_string())
        );
        assert_eq!(
            DataType::Hex.from_string("6869").unwrap(),
            Data::Hex("6869".to_string())
        );
        assert_eq!(
            DataType::Base58.from_string("8wr").unwrap(),
            Data::Base58("8wr".to_string())
        );
        assert_eq!(
            DataType::Base64.from_string("aGk=").unwrap(),
            Data::Base64("aGk=".to_string())
        );
        assert_eq!(
            DataType::EncryptionKey.from_string("6869").unwrap(),
            Data::Binary(b"hi".to_vec())
        );

        assert_eq!(DataType::from_str("hex").unwrap(), DataType::Hex);
        assert!(DataType::from_str("unknown").is_err());
    }

    #[test]
    fn fingerprint_generation_and_uniqueness() {
        let data = b"secret";
        let fp = DataFingerprint::get_fingerprint(data);
        let fp2 = DataFingerprint::get_fingerprint(data);
        assert_eq!(fp, fp2);

        let existing = vec![DataFingerprint {
            fingerprint: fp,
            probe: 0,
        }];
        let unique = DataFingerprint::find_unique(data, &existing);
        assert_eq!(unique.fingerprint, fp);
        assert_eq!(unique.probe, 1);

        let fp_str = DataFingerprint::from(fp).to_string();
        let parsed = DataFingerprint::from_str(&fp_str).unwrap();
        assert_eq!(parsed.fingerprint, fp);
    }

    #[test]
    fn formatted_data_encode_decode() {
        let fd = FormattedData::new(
            "name".to_string(),
            Data::Plain("value".to_string()),
            DataType::Plain,
            None,
        );
        let encoded = fd.encode().unwrap();
        let decoded = FormattedData::decode(&encoded).unwrap();
        assert_eq!(decoded.name, "name");
        assert_eq!(decoded.data, Data::Plain("value".to_string()));
        assert_eq!(decoded.data_type, DataType::Plain);
        assert_eq!(decoded.password_fingerprint, None);

        let fp_hex = "cafebabe";
        let data = "7777";
        let encoded_with_fp = format!(
            "name = data:application/vnd.binqbit.svpi;fp={};base64,{}",
            fp_hex, data
        );
        let decoded = FormattedData::decode(&encoded_with_fp).unwrap();
        assert_eq!(decoded.name, "name");
        assert_eq!(decoded.data_type, DataType::Base64);
        assert_eq!(decoded.data, Data::Hex(data.to_string()));
        assert_eq!(
            decoded.password_fingerprint,
            Some(DataFingerprint::from_str(fp_hex).unwrap().fingerprint)
        );
    }

    #[test]
    fn datainfo_new_sets_fields() {
        let data1 = b"foo";
        let info1 = DataInfo::new("name", 1, data1, DataType::Plain, None, &[]);
        assert_eq!(&info1.name[..4], b"name");
        assert_eq!(info1.address, 1);
        assert_eq!(info1.size, data1.len() as u32);
        assert_eq!(info1.data_type, DataType::Plain);

        let info2 = DataInfo::new(
            "name2",
            2,
            data1,
            DataType::Plain,
            None,
            &[info1.fingerprint],
        );
        assert_eq!(info2.fingerprint.fingerprint, info1.fingerprint.fingerprint);
        assert_eq!(info2.fingerprint.probe, 1);
    }
}
