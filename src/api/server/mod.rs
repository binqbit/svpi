use std::sync::{Arc, RwLock};

use rocket::{config::Config, figment::Profile, Build, Rocket};
use rocket_cors::CorsOptions;

use crate::{api::seg_mgmt::{start_connection_checking, DeviceStatus}, utils::args};

mod routes;

pub fn api_server() -> Rocket<Build> {
    println!("Starting SVPI Server on 0.0.0.0:3333");

    let config = Config {
        profile: Profile::new("api"),
        address: "0.0.0.0".parse().unwrap(),
        port: 3333,
        workers: 1,
        max_blocking: 1,
        ..Default::default()
    };

    let cors = CorsOptions::default()
        .to_cors()
        .expect("Error creating CORS options");

    let seg_mgmt = Arc::new(RwLock::new(DeviceStatus::connect_device()));

    if args::get_flag(vec!["--auto-exit", "-ae"]).is_some() {
        start_connection_checking(seg_mgmt.clone());
    }

    rocket::custom(config)
        .attach(cors)
        .manage(seg_mgmt)
        .mount("/", routes::route())
}
