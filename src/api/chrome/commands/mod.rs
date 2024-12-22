use get_data::GetDataRequest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum CommandRequest {
    #[serde(rename = "status")]
    Status {},
    #[serde(rename = "list")]
    List {},
    #[serde(rename = "get_data")]
    GetData(GetDataRequest),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "ok")]
    Ok,
    #[serde(rename = "device_error")]
    DeviceError,
    #[serde(rename = "device_not_found")]
    DeviceNotFound,
    #[serde(rename = "password_error")]
    PasswordError,
    #[serde(rename = "data_not_found")]
    DataNotFound,
    #[serde(rename = "error_read_data")]
    ReadDataError,
    #[serde(rename = "password_not_provided")]
    PasswordNotProvided,
}

pub mod status;
pub mod list;
pub mod get_data;
