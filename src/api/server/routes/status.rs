use rocket::{get, serde::json::Json, State};

use crate::{api::server::ApiState, protocol::api, utils::response::SvpiResponse};

#[get("/status")]
pub fn status(state: &State<ApiState>) -> Json<SvpiResponse> {
    let state = state.inner();
    let _guard = state.lock.lock().expect("Failed to lock API mutex");
    Json(api::status(
        api::ApiTransport::Server,
        state.interface_type.clone(),
    ))
}
