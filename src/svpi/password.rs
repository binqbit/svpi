use arboard::Clipboard;

use crate::{
    pass_mgr::PasswordManager,
    utils::{args, terminal},
};

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

    let password = terminal::get_password(None).expect("password");

    pass_mgr
        .add_encryption_key(&master_password, &name, &password)
        .expect("Failed to add encryption key");

    println!("Encryption key '{}' added successfully!", name);
}
