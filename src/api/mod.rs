use rocket::{config::Config, figment::Profile, Build, Rocket};
use rocket_cors::CorsOptions;

mod seg_mgmt;
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

    rocket::custom(config)
        .attach(cors)
        .mount("/", routes::route())
}
