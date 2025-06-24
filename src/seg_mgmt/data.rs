use base58::{FromBase58, ToBase58};
use base64::Engine;
use serde::Serialize;

use crate::{
    seg_mgmt::{Segment, SegmentError},
    utils::crypto::{decrypt, encrypt},
};

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum DataType {
    #[serde(rename = "binary")]
    Binary,
    #[serde(rename = "plain")]
    Plain,
    #[serde(rename = "hex")]
    Hex,
    #[serde(rename = "base64")]
    Base64,
    #[serde(rename = "base58")]
    Base58,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub enum Data {
    #[serde(rename = "encrypted")]
    Encrypted(Vec<u8>),
    #[serde(rename = "binary")]
    Binary(Vec<u8>),
    #[serde(rename = "plain")]
    Plain(String),
    #[serde(rename = "hex")]
    Hex(String),
    #[serde(rename = "base64")]
    Base64(String),
    #[serde(rename = "base58")]
    Base58(String),
}

pub struct FormattedData {
    pub name: String,
    pub data: Data,
    pub data_type: DataType,
}

#[derive(Debug)]
pub enum DataError {
    Utf8DecodeError(std::string::FromUtf8Error),
    HexDecodeError(hex::FromHexError),
    Base64DecodeError(base64::DecodeError),
    Base58DecodeError(base58::FromBase58Error),
    EncryptionError(ring::error::Unspecified),
    DecryptionError,
    NotString,
    InvalidData,
}

impl Data {
    pub fn to_bytes(&self, password: Option<&str>) -> Result<Vec<u8>, DataError> {
        let data = match self {
            Data::Encrypted(data) => Ok(data.clone()),
            Data::Binary(data) => Ok(data.clone()),
            Data::Plain(data) => Ok(data.as_bytes().to_vec()),
            Data::Hex(data) => hex::decode(data).map_err(DataError::HexDecodeError),
            Data::Base64(data) => base64::engine::general_purpose::STANDARD
                .decode(&data)
                .map_err(DataError::Base64DecodeError),
            Data::Base58(data) => data.from_base58().map_err(DataError::Base58DecodeError),
        }?;

        if let Some(password) = password {
            let encrypted_data =
                encrypt(&data, password.as_bytes()).map_err(DataError::EncryptionError)?;
            Ok(encrypted_data)
        } else {
            Ok(data)
        }
    }

    pub fn get_type(&self) -> DataType {
        match self {
            Data::Encrypted(_) => DataType::Binary,
            Data::Binary(_) => DataType::Binary,
            Data::Plain(_) => DataType::Plain,
            Data::Hex(_) => DataType::Hex,
            Data::Base64(_) => DataType::Base64,
            Data::Base58(_) => DataType::Base58,
        }
    }

    pub fn convert_to_type(&self, data_type: DataType) -> Result<Data, DataError> {
        let data = self.to_bytes(None)?;
        data_type.from_bytes(&data, false, None)
    }

    pub fn to_string_typed(&self, data_type: DataType) -> Result<String, DataError> {
        let data = self.convert_to_type(data_type)?;
        match data {
            Data::Encrypted(_) => Err(DataError::NotString),
            Data::Binary(_) => Err(DataError::NotString),
            Data::Plain(text) => Ok(text),
            Data::Hex(hex_str) => Ok(hex_str),
            Data::Base64(base64_str) => Ok(base64_str),
            Data::Base58(base58_str) => Ok(base58_str),
        }
    }

    pub fn to_string(&self) -> Result<String, DataError> {
        let data_type = self.get_type();
        let data_type = match data_type {
            DataType::Binary => DataType::Hex,
            _ => data_type,
        };
        self.to_string_typed(data_type)
    }

    pub fn detect_type(str: &str) -> Data {
        if Data::Hex(str.to_string()).to_bytes(None).is_ok() {
            Data::Hex(str.to_string())
        } else if Data::Base58(str.to_string()).to_bytes(None).is_ok() {
            Data::Base58(str.to_string())
        } else if Data::Base64(str.to_string()).to_bytes(None).is_ok() {
            Data::Base64(str.to_string())
        } else {
            Data::Plain(str.to_string())
        }
    }
}

impl DataType {
    pub fn from_bytes(
        &self,
        data: &[u8],
        is_encrypted: bool,
        password: Option<&str>,
    ) -> Result<Data, DataError> {
        let data = if is_encrypted {
            if let Some(pass) = password {
                match decrypt(&data, pass.as_bytes()) {
                    Ok(encrypted_data) => encrypted_data,
                    Err(_) => return Ok(Data::Encrypted(data.to_vec())),
                }
            } else {
                return Ok(Data::Encrypted(data.to_vec()));
            }
        } else {
            data.to_vec()
        };

        match self {
            DataType::Binary => Ok(Data::Binary(data)),
            DataType::Plain => String::from_utf8(data)
                .map(Data::Plain)
                .map_err(DataError::Utf8DecodeError),
            DataType::Hex => Ok(Data::Hex(hex::encode(&data))),
            DataType::Base64 => Ok(Data::Base64(
                base64::engine::general_purpose::STANDARD.encode(&data),
            )),
            DataType::Base58 => Ok(Data::Base58(data.to_base58())),
        }
    }

    pub fn from_string(&self, data: &str) -> Result<Data, DataError> {
        match self {
            DataType::Plain => Ok(Data::Plain(data.to_string())),
            DataType::Hex => Ok(Data::Hex(data.to_string())),
            DataType::Base64 => Ok(Data::Base64(data.to_string())),
            DataType::Base58 => Ok(Data::Base58(data.to_string())),
            _ => Err(DataError::NotString),
        }
    }
}

impl FormattedData {
    pub fn new(name: String, data: Data, data_type: DataType) -> Self {
        FormattedData {
            name,
            data,
            data_type,
        }
    }

    pub fn encode(&self) -> Result<String, DataError> {
        match &self.data {
            Data::Encrypted(_) => {
                let data = self.data.to_string_typed(DataType::Hex)?;
                Ok(format!("@{:?}/{}: {}", self.data_type, self.name, data))
            }
            Data::Binary(_) => {
                let data = self.data.to_string_typed(DataType::Hex)?;
                Ok(format!("{:?}/{}: {}", self.data_type, self.name, data))
            }
            Data::Plain(text) => Ok(format!("{:?}/{}: {}", self.data_type, self.name, text)),
            Data::Hex(hex_str) => Ok(format!("{:?}/{}: {}", self.data_type, self.name, hex_str)),
            Data::Base64(base64_str) => Ok(format!(
                "{:?}/{}: {}",
                self.data_type, self.name, base64_str
            )),
            Data::Base58(base58_str) => Ok(format!(
                "{:?}/{}: {}",
                self.data_type, self.name, base58_str
            )),
        }
    }

    pub fn decode(data: &str, password: Option<&str>) -> Result<Self, DataError> {
        let parts: Vec<&str> = data.split(":").collect();
        if parts.len() != 2 {
            return Err(DataError::InvalidData);
        }

        let type_name_parts = parts[0]
            .trim_start_matches('@')
            .split('/')
            .collect::<Vec<&str>>();
        if type_name_parts.len() != 2 {
            return Err(DataError::InvalidData);
        }

        let is_encrypted = parts[0].starts_with('@');
        let type_part = type_name_parts[0];
        let name = type_name_parts[1].to_string();
        let data = parts[1].trim().to_string();

        let from_type = match type_part {
            "binary" => Data::Hex(data),
            "plain" => Data::Plain(data),
            "hex" => Data::Hex(data),
            "base64" => Data::Base64(data),
            "base58" => Data::Base58(data),
            _ => return Err(DataError::NotString),
        };

        let to_type = match type_part {
            "binary" => DataType::Binary,
            "plain" => DataType::Plain,
            "hex" => DataType::Hex,
            "base64" => DataType::Base64,
            "base58" => DataType::Base58,
            _ => return Err(DataError::NotString),
        };

        let data = from_type.to_bytes(None)?;
        let data = to_type.from_bytes(&data, is_encrypted, password)?;

        Ok(FormattedData::new(name, data, to_type))
    }
}

impl Segment {
    pub fn to_formatted_data(
        &mut self,
        password: Option<&str>,
    ) -> Result<FormattedData, SegmentError> {
        let data = self.read_data(password)?;
        Ok(FormattedData::new(self.get_name(), data, self.data_type))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PASSWORD: &str = "secret";

    #[test]
    fn test_data_to_bytes_roundtrip() {
        let txt = "hello";

        // Plain
        let plain = Data::Plain(txt.to_string());
        assert_eq!(plain.to_bytes(None).unwrap(), txt.as_bytes());

        // Hex
        let hex = Data::Hex(hex::encode(txt));
        assert_eq!(hex.to_bytes(None).unwrap(), txt.as_bytes());

        // Base64
        let b64 = Data::Base64(base64::engine::general_purpose::STANDARD.encode(txt));
        assert_eq!(b64.to_bytes(None).unwrap(), txt.as_bytes());

        // Base58
        let b58 = Data::Base58(txt.as_bytes().to_base58());
        assert_eq!(b58.to_bytes(None).unwrap(), txt.as_bytes());

        // Binary
        let bin = Data::Binary(txt.as_bytes().to_vec());
        assert_eq!(bin.to_bytes(None).unwrap(), txt.as_bytes());
    }

    #[test]
    fn test_data_encryption_roundtrip() {
        let bin = Data::Binary(b"hello".to_vec());
        let encrypted = bin.to_bytes(Some(PASSWORD)).unwrap();
        assert_ne!(encrypted, b"hello".to_vec());

        let decrypted = DataType::Binary
            .from_bytes(&encrypted, true, Some(PASSWORD))
            .unwrap();
        assert_eq!(decrypted, bin);

        // wrong password should return encrypted data
        let wrong = DataType::Binary
            .from_bytes(&encrypted, true, Some("wrong"))
            .unwrap();
        assert!(matches!(wrong, Data::Encrypted(_)));
    }

    #[test]
    fn test_convert_between_types() {
        let plain = Data::Plain("hi".to_string());
        let hex = plain.convert_to_type(DataType::Hex).unwrap();
        assert_eq!(hex, Data::Hex("6869".to_string()));

        let base64 = hex.convert_to_type(DataType::Base64).unwrap();
        assert_eq!(base64, Data::Base64("aGk=".to_string()));

        let bin = base64.convert_to_type(DataType::Binary).unwrap();
        assert_eq!(bin, Data::Binary(b"hi".to_vec()));
    }

    #[test]
    fn test_to_string_methods() {
        let bin = Data::Binary(b"hi".to_vec());
        assert_eq!(bin.to_string().unwrap(), "6869");

        let plain = Data::Plain("hello".to_string());
        assert_eq!(plain.to_string().unwrap(), "hello");

        let base58 = Data::Base58(b"hi".to_base58());
        assert_eq!(base58.to_string().unwrap(), "8wr");
    }

    #[test]
    fn test_detect_type() {
        assert!(matches!(Data::detect_type("6869"), Data::Hex(_)));
        assert!(matches!(Data::detect_type("aGk="), Data::Base64(_)));
        assert!(matches!(Data::detect_type("8wr"), Data::Base58(_)));
        assert!(matches!(Data::detect_type("hello"), Data::Plain(_)));
    }

    #[test]
    fn test_formatted_data_encode_decode() {
        let fd = FormattedData::new(
            "name".to_string(),
            Data::Plain("value".to_string()),
            DataType::Plain,
        );
        let encoded = fd.encode().unwrap();
        assert_eq!(encoded, "Plain/name: value");

        let decoded = FormattedData::decode(&encoded.to_lowercase(), None).unwrap();
        assert_eq!(decoded.name, fd.name);
        assert_eq!(decoded.data, fd.data);
        assert_eq!(decoded.data_type, fd.data_type);

        // Encrypted variant
        let encrypted = Data::Binary(b"secret".to_vec())
            .to_bytes(Some(PASSWORD))
            .unwrap();
        let encoded = FormattedData::new(
            "enc".to_string(),
            Data::Encrypted(encrypted.clone()),
            DataType::Binary,
        )
        .encode()
        .unwrap();

        let decoded = FormattedData::decode(&encoded.to_lowercase(), Some(PASSWORD)).unwrap();
        assert_eq!(decoded.name, "enc");
        assert_eq!(decoded.data_type, DataType::Binary);
        assert_eq!(decoded.data, Data::Binary(b"secret".to_vec()));
    }
}
