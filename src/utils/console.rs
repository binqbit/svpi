use std::io::Write;

use rpassword::read_password;

use super::args;

pub fn confirm(text: &str) -> bool {
    print!("{} (y/N): ", text);
    std::io::stdout().flush().unwrap();
    let mut confirm = String::new();
    std::io::stdin().read_line(&mut confirm).expect("Failed to read answer");
    confirm.trim() == "y"
}

pub fn get_password(check_flag: bool, confirm: bool, text: Option<String>) -> Option<String> {
    let text = text.unwrap_or("Password".to_string());
    let password_flag = args::get_flag(vec!["--password", "-p", "--password2", "-p2"]);
    if !check_flag || password_flag.is_some() {
        loop {
            print!("{text}: ");
            std::io::stdout().flush().unwrap();
            let password = read_password().unwrap().trim().to_string();
            if password.is_empty() {
                return None;
            }
            if confirm || password_flag == Some("--password2".to_string()) || password_flag == Some("-p2".to_string()) {
                print!("Confirm {text}: ");
                std::io::stdout().flush().unwrap();
                let confirm_password = read_password().unwrap().trim().to_string();
                if !confirm_password.is_empty() {
                    if password != confirm_password {
                        println!("Passwords do not match!");
                        println!("Please try again.");
                        continue;
                    }
                }
            }
            return Some(password);
        }
    } else {
        return None;
    }
}