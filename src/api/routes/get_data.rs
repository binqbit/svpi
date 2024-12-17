use rocket::{get, request::{FromRequest, Outcome, Request}, response::content::RawJson};
use serde_json::{json, Value};
use crate::{api::seg_mgmt::DeviceStatus, seg_mgmt::DataType, svpi::get_password_for_decode, utils::crypto::decrypt};

pub struct GetQueryParams {
    name: String,
    password: Option<String>,
    use_root_password: Option<bool>,
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for GetQueryParams {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let name = request.query_value::<String>("name").and_then(|res| res.ok()).unwrap_or_default();
        let password = request.query_value::<String>("password").and_then(|res| res.ok());
        let use_root_password = request.query_value::<bool>("use_root_password").and_then(|res| res.ok());
        Outcome::Success(GetQueryParams { name, password, use_root_password })
    }
}

#[get("/get")]
pub fn get_data(params: GetQueryParams) -> RawJson<Value> {
    let seg_mgmt = DeviceStatus::connect_device();
    match seg_mgmt {
        DeviceStatus::Some(mut seg_mgmt) => {
            let res = match &params.password {
                Some(password) => get_password_for_decode(&mut seg_mgmt, password, params.use_root_password.unwrap_or(false)),
                None => Ok(None),
            };
            match res {
                Err(err) => {
                    println!("[API::Get] Error decoding root password: {}", err);
                    return RawJson(json!({"error": "password_error"}))
                },
                Ok(password) => {
                    let seg = seg_mgmt.find_segment_by_name(&params.name);
                    let seg = match seg {
                        Some(seg) => seg.clone(),
                        None => {
                            println!("[API::Get] Data not found: {}", params.name);
                            return RawJson(json!({"error": "data_not_found"}))
                        },
                    };
                    let data = match seg_mgmt.read_segment_data(&seg) {
                        Ok(data) => data,
                        Err(err) => {
                            println!("[API::Get] Error reading data: {}", err);
                            return RawJson(json!({"error": "error_read_data"}))
                        },
                    };
                    let decoded = match seg.data_type {
                        DataType::Encrypted => {
                            let password = match password {
                                Some(password) => password,
                                None => {
                                    println!("[API::Get] Password not provided for decryption");
                                    return RawJson(json!({"error": "password_not_provided"}))
                                },
                            };
                            match decrypt(&data, password.as_bytes()) {
                                Ok(data) => String::from_utf8_lossy(&data).into_owned(),
                                Err(err) => {
                                    println!("[API::Get] Error decrypting data: {}", err);
                                    return RawJson(json!({"error": "password_error"}))
                                },
                            }
                        },
                        DataType::Plain => String::from_utf8_lossy(&data).into_owned(),
                    };
                    println!("[API::Get] Data found: {}", seg.get_name());
                    RawJson(json!({"name": seg.get_name(), "data": decoded}))
                }
            }
        },
        DeviceStatus::DeviceNotFound => {
            println!("[API::Get] Device not found");
            RawJson(json!({"error": "device_not_found"}))
        },
        DeviceStatus::DeviceError => {
            println!("[API::Get] Error loading segments");
            RawJson(json!({"error": "device_error"}))
        },
    }
}