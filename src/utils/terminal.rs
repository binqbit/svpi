use std::io::Write;

use rpassword::read_password;

use super::args;

pub fn confirm(text: &str) -> bool {
    print!("{} (y/N): ", text);
    std::io::stdout().flush().unwrap();
    let mut confirm = String::new();
    std::io::stdin()
        .read_line(&mut confirm)
        .expect("Failed to read answer");
    confirm.trim() == "y"
}

pub fn get_password(check_flag: bool, confirm: bool, text: Option<String>) -> Option<String> {
    let text = text.unwrap_or("Password".to_string());
    let password_flag = args::check_flag(vec![
        "--password",
        "-p",
        "--root-password",
        "-rp",
        "--password2",
        "-p2",
        "--root-password2",
        "-rp2",
    ]);
    let is_confirm_password = confirm || args::check_flag(vec!["--confirm", "-cp"]);

    if !check_flag || password_flag {
        loop {
            print!("{text}: ");
            std::io::stdout().flush().unwrap();
            let password = read_password().unwrap().trim().to_string();
            if password.is_empty() {
                return None;
            }

            if is_confirm_password {
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
