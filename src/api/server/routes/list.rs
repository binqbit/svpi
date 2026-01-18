use crate::{api::server::ApiState, protocol::api, utils::response::SvpiResponse};
use rocket::{get, serde::json::Json, State};

#[get("/list")]
pub fn list(state: &State<ApiState>) -> Json<SvpiResponse> {
    let state = state.inner();
    let _guard = state.lock.lock().expect("Failed to lock API mutex");
    Json(api::list(
        api::ApiTransport::Server,
        state.interface_type.clone(),
    ))
}
