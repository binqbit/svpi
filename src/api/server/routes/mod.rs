use rocket::{routes, Route};

mod get_data;
mod list;
mod status;

pub fn route() -> Vec<Route> {
    routes![status::status, list::list, get_data::get_data]
}
