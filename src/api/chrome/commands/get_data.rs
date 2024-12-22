use serde::{Deserialize, Serialize};
use crate::{seg_mgmt::{DataType, SegmentManager}, svpi::get_password_for_decode, utils::crypto::decrypt};

use super::Status;

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDataRequest {
    name: String,
    password: Option<String>,
    use_root_password: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetDataResponse {
    status: Status,
    name: Option<String>,
    data: Option<String>,
}

pub fn get_data(params: GetDataRequest, seg_mgmt: &mut SegmentManager) -> Result<GetDataResponse, Status> {
    let res = match &params.password {
        Some(password) => get_password_for_decode(seg_mgmt, password, params.use_root_password.unwrap_or(false)),
        None => Ok(None),
    };
    match res {
        Err(_) => {
            return Err(Status::PasswordError);
        },
        Ok(password) => {
            let seg = seg_mgmt.find_segment_by_name(&params.name);
            let seg = match seg {
                Some(seg) => seg.clone(),
                None => {
                    return Err(Status::DataNotFound);
                },
            };
            let data = match seg_mgmt.read_segment_data(&seg) {
                Ok(data) => data,
                Err(_) => {
                    return Err(Status::ReadDataError);
                },
            };
            let decoded = match seg.data_type {
                DataType::Encrypted => {
                    let password = match password {
                        Some(password) => password,
                        None => {
                            return Err(Status::PasswordNotProvided);
                        },
                    };
                    match decrypt(&data, password.as_bytes()) {
                        Ok(data) => String::from_utf8_lossy(&data).into_owned(),
                        Err(_) => {
                            return Err(Status::PasswordError);
                        },
                    }
                },
                DataType::Plain => String::from_utf8_lossy(&data).into_owned(),
            };
            Ok(GetDataResponse {
                status: Status::Ok,
                name: Some(seg.get_name()),
                data: Some(decoded),
            })
        }
    }
}