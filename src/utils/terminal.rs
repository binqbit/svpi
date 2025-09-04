use std::io::Write;

use rpassword::read_password;

pub fn confirm(msg: &str) -> bool {
    print!("{} (y/N): ", msg);
    std::io::stdout().flush().unwrap();

    let mut confirm = String::new();
    std::io::stdin()
        .read_line(&mut confirm)
        .expect("Failed to read answer");

    confirm.to_lowercase().trim() == "y"
}

pub fn get_password(title: Option<&str>) -> Option<String> {
    let title = title.unwrap_or("password");

    loop {
        print!("Enter {title}: ");
        std::io::stdout().flush().unwrap();

        let password = read_password()
            .expect("Failed to read password")
            .trim()
            .to_string();

        if password.is_empty() {
            return None;
        }

        print!("Confirm {title}: ");
        std::io::stdout().flush().unwrap();

        let confirm_password = read_password()
            .expect("Failed to read password")
            .trim()
            .to_string();

        if !confirm_password.is_empty() {
            if password != confirm_password {
                println!("Passwords do not match!");
                println!("Please try again.");
                continue;
            }
        }

        return Some(password);
    }
}
