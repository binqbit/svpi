use crate::{
    pass_mgr::PasswordManager,
    seg_mgr::EncryptionLevel,
    utils::{args, terminal},
};
use arboard::Clipboard;
use std::str::FromStr;

pub fn set_master_password() {
    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    let master_password =
        if let Some(master_password) = terminal::get_password(Some("master password")) {
            master_password
        } else {
            let mut clipboard = Clipboard::new().expect("Failed to create clipboard instance!");
            clipboard
                .get_text()
                .expect("Failed to get text from clipboard!")
        };

    pass_mgr
        .set_master_password(&master_password)
        .expect("Failed to set master password");

    println!("Master password set successfully!");
}

pub fn reset_master_password() {
    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    if !pass_mgr.is_master_password_set() {
        println!("Master password not set!");
        return;
    }

    if !terminal::confirm("Are you sure you want to remove the master password?") {
        return;
    }

    pass_mgr
        .reset_master_password()
        .expect("Failed to reset master password");
}

pub fn check_master_password() {
    let pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    if !pass_mgr.is_master_password_set() {
        println!("Master password not set!");
        return;
    }

    let master_password =
        if let Some(master_password) = terminal::get_password(Some("master password")) {
            master_password
        } else {
            let mut clipboard = Clipboard::new().expect("Failed to create clipboard instance!");
            clipboard
                .get_text()
                .expect("Failed to get text from clipboard!")
        };

    let is_valid = pass_mgr.check_master_password(&master_password);

    if is_valid {
        println!("Master password is valid.");
    } else {
        println!("Master password is invalid.");
    }
}

pub fn add_encryption_key() {
    let name = args::get_param_by_id(0).expect("Name is required");
    let level =
        EncryptionLevel::from_str(&args::get_param_by_id(1).unwrap_or(String::from("medium")))
            .expect("Invalid encryption level");

    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    if !pass_mgr.is_master_password_set() {
        println!("Master password not set!");
        return;
    }

    let master_password =
        if let Some(master_password) = terminal::get_password(Some("master password")) {
            master_password
        } else {
            let mut clipboard = Clipboard::new().expect("Failed to create clipboard instance!");
            clipboard
                .get_text()
                .expect("Failed to get text from clipboard!")
        };

    if !pass_mgr.check_master_password(&master_password) {
        println!("Master password is invalid.");
        return;
    } else {
        println!("Master password is valid.");
    }

    let password = terminal::get_password(None).expect("password");

    pass_mgr
        .add_encryption_key(&master_password, &name, &password, level)
        .expect("Failed to add encryption key");

    println!("Encryption key '{}' added successfully!", name);
}

pub fn link_key() {
    let name = args::get_param_by_id(0).expect("Name is required");

    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    let password = args::get_param_by_flag("--password")
        .or_else(|| terminal::get_password(None))
        .expect("Failed to get password");

    pass_mgr
        .link_key(&name, &password)
        .expect("Failed to link encryption key");

    println!("Encryption key '{}' linked successfully!", name);
}

pub fn sync_keys() {
    let mut pass_mgr =
        PasswordManager::load_from_args_or_default().expect("Failed to load password manager");

    let master_password =
        if let Some(master_password) = terminal::get_password(Some("master password")) {
            master_password
        } else {
            let mut clipboard = Clipboard::new().expect("Failed to create clipboard instance!");
            clipboard
                .get_text()
                .expect("Failed to get text from clipboard!")
        };

    if !pass_mgr.check_master_password(&master_password) {
        println!("Master password is invalid.");
        return;
    } else {
        println!("Master password is valid.");
    }

    pass_mgr
        .sync_encryption_keys(&master_password)
        .expect("Failed to sync encryption keys");

    println!("Encryption keys synced successfully!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{data_mgr::DataInterfaceType, seg_mgr::DataType};

    fn setup_mgr() -> PasswordManager {
        let mut mgr = PasswordManager::from_device_type(DataInterfaceType::Memory(vec![])).unwrap();
        mgr.get_data_manager().init_device(256).unwrap();
        mgr
    }

    #[test]
    fn master_password_flow() {
        let mut mgr = setup_mgr();
        assert!(!mgr.is_master_password_set());
        mgr.set_master_password("master").unwrap();
        assert!(mgr.is_master_password_set());
        assert!(mgr.check_master_password("master"));
        mgr.reset_master_password().unwrap();
        assert!(!mgr.is_master_password_set());
    }

    #[test]
    fn add_encryption_key_creates_segment() {
        let mut mgr = setup_mgr();
        mgr.set_master_password("master").unwrap();
        assert!(mgr
            .add_encryption_key("master", "key1", "pwd", EncryptionLevel::default())
            .unwrap());
        let seg = mgr.get_data_manager().find_segment_by_name("key1").unwrap();
        assert_eq!(seg.info.data_type, DataType::EncryptionKey);
    }
}
