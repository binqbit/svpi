use rocket::{form::FromForm, get, serde::json::Json, State};

use crate::{
    api::server::ApiState,
    protocol::api::{self, GetDataRequest},
    utils::response::SvpiResponse,
};

#[derive(FromForm)]
pub struct GetQueryParams {
    name: Option<String>,
    password: Option<String>,
}

#[get("/get?<params..>")]
pub fn get_data(state: &State<ApiState>, params: GetQueryParams) -> Json<SvpiResponse> {
    let state = state.inner();
    let _guard = state.lock.lock().expect("Failed to lock API mutex");
    Json(api::get_data(
        api::ApiTransport::Server,
        state.interface_type.clone(),
        GetDataRequest {
            name: params.name.unwrap_or_default(),
            password: params.password,
        },
    ))
}
