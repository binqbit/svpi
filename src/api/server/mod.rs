use std::sync::{Arc, Mutex};

use rocket::{config::Config, figment::Profile, Build, Rocket};
use rocket_cors::CorsOptions;

use crate::data_mgr::{DataInterfaceType, DataManager};

mod routes;

pub struct ApiState {
    pub interface_type: DataInterfaceType,
    pub lock: Arc<Mutex<()>>,
}

fn start_connection_checking(interface_type: DataInterfaceType, lock: Arc<Mutex<()>>) {
    std::thread::spawn(move || loop {
        std::thread::sleep(std::time::Duration::from_secs(1));
        let _guard = lock.lock().expect("Failed to lock API mutex");

        if !matches!(interface_type, DataInterfaceType::SerialPort) {
            continue;
        }

        let Ok(mut data_mgr) = interface_type.load_data_manager() else {
            std::process::exit(1);
        };

        match &mut data_mgr {
            DataManager::SerialPort(spdm) => {
                if let Ok(res) = spdm.test(b"test") {
                    if res.as_slice() == b"test" {
                        continue;
                    }
                }
                std::process::exit(1);
            }
            DataManager::FileSystem(_) | DataManager::Memory(_) => continue,
        }
    });
}

pub fn api_server(interface_type: DataInterfaceType, auto_exit: bool) -> Rocket<Build> {
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

    let lock = Arc::new(Mutex::new(()));

    if auto_exit {
        start_connection_checking(interface_type.clone(), lock.clone());
    }

    rocket::custom(config)
        .attach(cors)
        .manage(ApiState {
            interface_type,
            lock,
        })
        .mount("/", routes::route())
}
