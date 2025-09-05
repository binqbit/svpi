use std::str::FromStr;

use arboard::Clipboard;

use crate::{
    pass_mgr::PasswordManager,
    seg_mgr::{Data, DataType, DATA_FINGERPRINT_SIZE},
    utils::{args, terminal},
};

pub fn get_data_list() {
    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");
    let data_mgr = pass_mgr.get_data_manager();

    println!("{}", "=".repeat(36));
    println!("| {:14} | {:15} |", "Memory Size", "Value (bytes)");
    println!("{}", "=".repeat(36));
    println!("| {:14} | {:15} |", "Total", data_mgr.metadata.memory_size);
    println!("{}", "-".repeat(36));
    println!("| {:14} | {:15} |", "Free", data_mgr.free_memory_size());
    println!("{}", "-".repeat(36));

    let data_list = data_mgr.get_active_segments();
    if !data_list.is_empty() {
        println!("{}", "=".repeat(110));
        println!(
            "| {:32} | {:15} | {:25} | {:25} |",
            "Name", "Data Type", "Fingerprint", "Password Fingerprint"
        );
        println!("{}", "=".repeat(110));

        for data in data_list {
            let name = data.get_name();
            let data_type = data.info.data_type.to_string();
            let fingerprint = data.info.fingerprint.to_string();
            let password_fingerprint = if let Some(fingerprint) = data.info.password_fingerprint {
                Data::Binary(fingerprint.to_vec())
                    .to_string_typed(DataType::Hex)
                    .expect("Failed to convert fingerprint")
            } else {
                String::from("-".repeat(DATA_FINGERPRINT_SIZE * 2))
            };

            println!(
                "| {:32} | {:15} | {:25} | {:25} |",
                name, data_type, fingerprint, password_fingerprint
            );
            println!("{}", "-".repeat(110));
        }
    } else {
        println!("No data found!");
    }
}

pub fn save_data() {
    let name = args::get_param_by_id(0).expect("Name is required");
    let data = if let Some(data) = args::get_param_by_id(1) {
        data
    } else {
        let mut clipboard = Clipboard::new().expect("Failed to create clipboard instance!");
        clipboard
            .get_text()
            .expect("Failed to get text from clipboard!")
    };
    let encryption_key = terminal::get_password(None);

    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    let is_saved = pass_mgr
        .save_password(&name, &data, encryption_key)
        .expect("Failed to save data");

    if is_saved {
        println!("Data '{}' saved!", name);
    } else {
        println!("Not enough memory!");
        println!("Please optimize the memory: svpi optimize");
    }
}

pub fn get_data() {
    let name = args::get_param_by_id(0).expect("Name is required");

    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    let data = pass_mgr
        .read_password(&name, || {
            terminal::get_password(None).expect("Failed to get password")
        })
        .expect("Failed to read data");

    if args::check_flags(&["--clipboard", "-c"]) {
        let mut clipboard = Clipboard::new().expect("Failed to create clipboard instance!");
        clipboard
            .set_text(data)
            .expect("Failed to set text to clipboard");
        println!("Data copied to clipboard!");
    } else {
        println!("Data: {}", data);
    }
}

pub fn remove_data() {
    let name = args::get_param_by_id(0).expect("Name is required");

    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    let is_existed = pass_mgr
        .get_data_manager()
        .find_segment_by_name(&name)
        .is_some();
    if !is_existed {
        println!("Data '{}' not found!", name);
        return;
    }

    if terminal::confirm(&format!("Are you sure you want to remove '{}'?", name)) {
        pass_mgr
            .remove_password(&name)
            .expect("Failed to remove data");
        println!("Data '{}' removed!", name);
    }
}

pub fn rename_data() {
    let old_name = args::get_param_by_id(0).expect("Old name is required");
    let new_name = args::get_param_by_id(1).expect("New name is required");

    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    pass_mgr
        .rename_password(&old_name, &new_name)
        .expect("Failed to rename data");
    println!("Data '{}' renamed to '{}'", old_name, new_name);
}

pub fn change_data_type() {
    let name = args::get_param_by_id(0).expect("Name is required");
    let new_data_type =
        DataType::from_str(&args::get_param_by_id(1).expect("New data type is required"))
            .expect("Invalid data type");

    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    pass_mgr
        .change_data_type(&name, new_data_type)
        .expect("Failed to change data type");

    println!("Data '{}' changed to type '{:?}'", name, new_data_type);
}

pub fn change_password() {
    let name = args::get_param_by_id(0).expect("Name is required");

    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    let data = pass_mgr
        .read_password(&name, || {
            terminal::get_password(None).expect("Failed to get password")
        })
        .expect("Failed to read data");

    let new_password = terminal::get_password(Some("new password"));

    pass_mgr
        .save_password(&name, &data, new_password)
        .expect("Failed to change password");

    println!("Password for data '{}' changed!", name);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data_mgr::DataInterfaceType, seg_mgr::DataType};

    fn setup_mgr() -> PasswordManager {
        let mut mgr =
            PasswordManager::from_device_type(DataInterfaceType::Memory(vec![])).expect("init");
        mgr.get_data_manager()
            .init_device(1024)
            .expect("init device");
        mgr
    }

    #[test]
    fn save_and_list_data() {
        let mut mgr = setup_mgr();
        mgr.save_password("first", "alpha", None).unwrap();
        mgr.save_password("second", "beta", None).unwrap();

        let names: Vec<_> = mgr
            .get_data_manager()
            .get_active_segments()
            .iter()
            .map(|s| s.get_name())
            .collect();
        assert!(names.contains(&"first".to_string()));
        assert!(names.contains(&"second".to_string()));
    }

    #[test]
    fn rename_change_type_and_remove() {
        let mut mgr = setup_mgr();
        mgr.save_password("item", "42", None).unwrap();
        mgr.rename_password("item", "renamed").unwrap();
        mgr.change_data_type("renamed", DataType::Hex).unwrap();

        let seg = mgr
            .get_data_manager()
            .find_segment_by_name("renamed")
            .unwrap();
        assert_eq!(seg.info.data_type, DataType::Hex);

        mgr.remove_password("renamed").unwrap();
        assert!(mgr
            .get_data_manager()
            .find_segment_by_name("renamed")
            .is_none());
    }
}
