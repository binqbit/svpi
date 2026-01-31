use std::io::{IsTerminal, Write};

use rpassword::read_password;
use rustyline::{error::ReadlineError, DefaultEditor};

pub enum ReplReader {
    Rustyline(DefaultEditor),
    Plain,
}

impl ReplReader {
    pub fn new() -> Self {
        if std::io::stdin().is_terminal() {
            match DefaultEditor::new() {
                Ok(v) => return Self::Rustyline(v),
                Err(err) => {
                    eprintln!("device_error: Failed to initialize interactive console: {err}");
                }
            }
        }

        Self::Plain
    }

    pub fn read_line(&mut self, prompt: &str) -> Result<Option<String>, String> {
        match self {
            ReplReader::Rustyline(rl) => match rl.readline(prompt) {
                Ok(v) => Ok(Some(v)),
                Err(ReadlineError::Eof) | Err(ReadlineError::Interrupted) => Ok(None),
                Err(err) => Err(err.to_string()),
            },
            ReplReader::Plain => {
                print!("{prompt}");
                std::io::stdout().flush().map_err(|err| err.to_string())?;

                let stdin = std::io::stdin();
                let mut line = String::new();
                match stdin.read_line(&mut line) {
                    Ok(0) => Ok(None),
                    Ok(_) => Ok(Some(line)),
                    Err(err) => Err(err.to_string()),
                }
            }
        }
    }

    pub fn add_history_entry(&mut self, line: &str) {
        if let ReplReader::Rustyline(rl) = self {
            let _ = rl.add_history_entry(line);
        }
    }

    pub fn clear_history(&mut self) {
        if let ReplReader::Rustyline(rl) = self {
            let _ = rl.clear_history();
        }
    }
}

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

    print!("Enter {title}: ");
    std::io::stdout().flush().unwrap();

    let password = read_password()
        .expect("Failed to read password")
        .trim()
        .to_string();

    if password.is_empty() {
        None
    } else {
        Some(password)
    }
}

pub fn get_password_confirmed(title: Option<&str>) -> Option<String> {
    let title = title.unwrap_or("password");

    loop {
        let Some(password) = get_password(Some(title)) else {
            return None;
        };

        print!("Confirm {title}: ");
        std::io::stdout().flush().unwrap();

        let confirm_password = read_password()
            .expect("Failed to read password")
            .trim()
            .to_string();

        if password == confirm_password {
            return Some(password);
        }

        println!("Passwords do not match!");
        println!("Please try again.");
    }
}
