use crate::{
    data_mgr::DataInterfaceType,
    pass_mgr::PasswordManager,
    seg_mgr::{Data, DataType, FormattedData, DATA_FINGERPRINT_SIZE},
    utils::{args, terminal},
};
use std::{
    fs::File,
    io::{Read, Write},
};

pub fn init_device() {
    let memory_size = args::get_param_by_id(0)
        .expect("Memory size is required")
        .parse::<u32>()
        .expect("Memory size must be a number");

    let interface_type = DataInterfaceType::from_args_or_default();
    let mut pass_mgr =
        PasswordManager::from_device_type(interface_type).expect("Failed to load password manager");

    let is_initialized = pass_mgr
        .get_data_manager()
        .check_init_data()
        .expect("Failed to check init data");
    if is_initialized {
        println!("Device already initialized!");
        println!("If you reinitialize the device, all data will be lost!");
    }

    let is_confirmed = terminal::confirm("Are you sure you want to initialize the device?");
    if is_confirmed {
        pass_mgr
            .get_data_manager()
            .init_device(memory_size)
            .expect("Failed to initialize device");
        println!("Device initialized!");
    }
}

pub fn check_device() {
    let interface_type = DataInterfaceType::from_args_or_default();
    let mut pass_mgr =
        PasswordManager::from_device_type(interface_type).expect("Failed to load password manager");

    let is_initialized = pass_mgr
        .get_data_manager()
        .check_init_data()
        .unwrap_or(false);

    if is_initialized {
        println!("Device is initialized.");
    } else {
        println!("Device is not initialized.");
    }
}

pub fn format_device() {
    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    let confirm = terminal::confirm("Are you sure you want to format the data?");
    if confirm {
        pass_mgr
            .get_data_manager()
            .format_data()
            .expect("Failed to format device");
        println!("Data formatted!");
    }
}

pub fn optimize_device() {
    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    let optimized_memory = pass_mgr
        .get_data_manager()
        .optimize_segments()
        .expect("Failed to optimize device");
    let seg_mgr = pass_mgr.get_data_manager();

    println!("{}", "=".repeat(36));
    println!("| {:14} | {:15} |", "Memory Size", "Value (bytes)");
    println!("{}", "=".repeat(36));
    println!("| {:14} | {:15} |", "Total", seg_mgr.metadata.memory_size);
    println!("{}", "-".repeat(36));
    println!("| {:14} | {:15} |", "Free", seg_mgr.free_memory_size());
    println!("{}", "-".repeat(36));
    println!("| {:14} | {:15} |", "Optimized", optimized_memory);
    println!("{}", "-".repeat(36));
}

pub fn export_data() {
    let file_path = args::get_param_by_id(0).expect("File name is required");

    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    let mut list = Vec::new();
    let mut passwords: Vec<([u8; DATA_FINGERPRINT_SIZE], Vec<u8>)> = Vec::new();

    let data_fingerprints = pass_mgr
        .get_data_manager()
        .get_active_segments()
        .into_iter()
        .filter(|seg| seg.info.data_type != DataType::EncryptionKey)
        .filter_map(|seg| seg.info.password_fingerprint.map(|fp| (seg.get_name(), fp)))
        .collect::<Vec<_>>();

    for (name, fingerprint) in data_fingerprints {
        let is_existing = passwords.iter().any(|(fp, _)| *fp == fingerprint);
        if !is_existing {
            if let Some(password) =
                terminal::get_password(Some(&format!("password for '{}'", name)))
            {
                let password = pass_mgr
                    .get_encryption_key(&password)
                    .expect("Failed to get encryption key");
                passwords.push(password);
            }
        }
    }

    let mut segments = pass_mgr
        .get_data_manager()
        .get_active_segments_mut()
        .into_iter()
        .filter(|seg| seg.info.data_type != DataType::EncryptionKey)
        .collect::<Vec<_>>();

    for seg in segments.iter_mut() {
        let password = if let Some(fingerprint) = seg.info.password_fingerprint {
            if let Some((_, password)) = passwords.iter().find(|(fp, _)| *fp == fingerprint) {
                Some(password.clone())
            } else {
                None
            }
        } else {
            None
        };

        let data = seg
            .read_data()
            .expect("Failed to read data")
            .to_bytes()
            .expect("Failed to convert data to bytes");

        let (data, password_fingerprint) = if let Some(password) = password {
            let result = seg.info.data_type.decrypt(&data, &password);
            match result {
                Ok(data) => (data, None),
                Err(_) => (Data::Binary(data), seg.info.password_fingerprint),
            }
        } else {
            (Data::Binary(data), seg.info.password_fingerprint)
        };

        let formatted_data = FormattedData::new(
            seg.get_name(),
            data,
            seg.info.data_type,
            password_fingerprint,
        )
        .encode()
        .expect("Failed to encode formatted data");

        list.push(formatted_data);
    }

    let str = list.join("\n");
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(str.as_bytes())
        .expect("Failed to write to file");

    println!("Data exported to '{}'", file_path);
}

pub fn import_data() {
    let file_path = args::get_param_by_id(0).expect("File name is required");

    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");
    let seg_mgmt = pass_mgr.get_data_manager();

    let str = std::fs::read_to_string(&file_path).expect("Failed to read file");
    let list = str.lines();

    for str in list {
        let formatted_data = FormattedData::decode(str).expect("Failed to decode formatted data");
        let data = formatted_data
            .data
            .to_bytes()
            .expect("Failed to convert data to bytes");

        let seg = seg_mgmt
            .set_segment(
                &formatted_data.name,
                &data,
                formatted_data.data_type,
                formatted_data.password_fingerprint,
            )
            .expect("Failed to set data");

        if seg.is_none() {
            println!("Not enough memory!");
            println!("Please optimize the memory: svpi optimize");
            break;
        }
    }

    println!("Data imported from '{}'", file_path);
}

pub fn save_dump() {
    let file_path = args::get_param_by_id(0).expect("File name is required");

    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    let data = pass_mgr
        .get_data_manager()
        .get_dump()
        .expect("Failed to get dump");
    let mut file = File::create(&file_path).expect("Failed to create file");
    file.write_all(&data).expect("Failed to write to file");

    println!("Dump saved to '{}'", file_path);
}

pub fn load_dump() {
    let file_path = args::get_param_by_id(0).expect("File name is required");

    let interface_type = DataInterfaceType::from_args_or_default();
    let mut pass_mgr =
        PasswordManager::from_device_type(interface_type).expect("Failed to load password manager");

    let is_initialized = pass_mgr
        .get_data_manager()
        .check_init_data()
        .expect("Failed to check init data");
    if is_initialized {
        println!("Device already initialized!");
        println!("If you load a dump, all data will be lost!");
    }

    let mut file = File::open(&file_path).expect("Failed to open file");
    let mut data = Vec::new();
    file.read_to_end(&mut data).expect("Failed to read file");

    let is_confirmed = terminal::confirm("Are you sure you want to load the dump?");
    if is_confirmed {
        pass_mgr
            .get_data_manager()
            .set_dump(&data)
            .expect("Failed to load dump");
        println!("Dump loaded!");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_mgr::DataInterfaceType;
    use std::fs;

    fn setup_mgr() -> PasswordManager {
        let mut mgr = PasswordManager::from_device_type(DataInterfaceType::Memory(vec![])).unwrap();
        mgr.get_data_manager().init_device(256).unwrap();
        mgr
    }

    #[test]
    fn init_and_format_cycle() {
        let mut mgr = setup_mgr();
        assert!(mgr.get_data_manager().check_init_data().unwrap());
        mgr.save_password("a", "1", None).unwrap();
        assert!(mgr.get_data_manager().find_segment_by_name("a").is_some());
        mgr.get_data_manager().format_data().unwrap();
        mgr.get_data_manager().load_segments().unwrap();
        assert!(mgr.get_data_manager().get_active_segments().is_empty());
    }

    #[test]
    fn dump_and_restore() {
        let mut mgr = setup_mgr();
        mgr.save_password("a", "1", None).unwrap();
        let dump = mgr.get_data_manager().get_dump().unwrap();
        let mut restored = PasswordManager::try_load(DataInterfaceType::Memory(dump)).unwrap();
        assert!(restored
            .get_data_manager()
            .find_segment_by_name("a")
            .is_some());
    }

    #[test]
    fn export_and_import_roundtrip() {
        let mut mgr = setup_mgr();
        mgr.save_password("one", "1", None).unwrap();
        mgr.save_password("two", "2", None).unwrap();

        let temp = std::env::temp_dir().join("svpi_export.txt");

        // export similar to export_data
        let mut list = Vec::new();
        for seg in mgr.get_data_manager().get_active_segments_mut() {
            let data = seg.read_data().unwrap();
            let formatted = FormattedData::new(
                seg.get_name(),
                data,
                seg.info.data_type,
                seg.info.password_fingerprint,
            )
            .encode()
            .unwrap();
            list.push(formatted);
        }
        fs::write(&temp, list.join("\n")).unwrap();

        // import back
        let mut mgr2 = setup_mgr();
        let contents = fs::read_to_string(&temp).unwrap();
        for line in contents.lines() {
            let formatted = FormattedData::decode(line).unwrap();
            let data = formatted.data.to_bytes().unwrap();
            mgr2.get_data_manager()
                .set_segment(
                    &formatted.name,
                    &data,
                    formatted.data_type,
                    formatted.password_fingerprint,
                )
                .unwrap();
        }

        assert!(mgr2
            .get_data_manager()
            .find_segment_by_name("one")
            .is_some());
        assert!(mgr2
            .get_data_manager()
            .find_segment_by_name("two")
            .is_some());

        let _ = fs::remove_file(temp);
    }
}
