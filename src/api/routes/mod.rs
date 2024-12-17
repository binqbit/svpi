use rocket::{ routes, Route };

mod status;
mod list;
mod get_data;

pub fn route() -> Vec<Route> {
    routes![
        status::status,
        list::list,
        get_data::get_data
    ]
}