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

pub fn get_password(check_flag: bool) -> Option<String> {
    if !check_flag {
        print!("Password: ");
        std::io::stdout().flush().unwrap();
        let password = read_password().unwrap().trim().to_string();
        if password.is_empty() {
            return None;
        }
        return Some(password);
    }
    
    let password_flag = args::get_flag(vec!["--password", "-p", "--password2", "-p2"]);
    if let Some(password_flag) = password_flag {
        loop {
            print!("Password: ");
            std::io::stdout().flush().unwrap();
            let password = read_password().unwrap().trim().to_string();
            if password.is_empty() {
                return None;
            }
            if let "--password2" | "-p2" = password_flag.as_str() {
                print!("Confirm password: ");
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