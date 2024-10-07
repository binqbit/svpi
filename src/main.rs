mod spdm;

use spdm::SerialPortDataManager;
use std::io::{self, Write};

fn main() -> std::io::Result<()> {
    let mut spdm = SerialPortDataManager::find_device();

    let data = spdm.test(b"Hello, World!")?;
    if data != b"Hello, World!" {
        eprintln!("Test connection failed: {:?}", std::str::from_utf8(&data).unwrap());
    } else {
        println!("Test connection successful.");
    }

    println!("Available commands:");
    println!("\t@ <new_address> - Set Address");
    println!("\t# <new_length>  - Set Length");

    let mut address = 0;
    let mut length = 100;

    loop {
        println!("{}", "-".repeat(34));
        println!("| Address: {address:5} | Length: {length:5} |");
        match spdm.read_data(address, length) {
            Ok(bytes_read) => {
                match std::str::from_utf8(&bytes_read) {
                    Ok(str) => {
                        let data = format!("{str:?}");
                        println!("{}", "-".repeat(if data.len() > 100 { 4 + data.len() } else { 104 }));
                        println!("| {data:100} |");
                        println!("{}", "-".repeat(if data.len() > 100 { 4 + data.len() } else { 104 }));
                    },
                    Err(err) => eprintln!("Error decoding data: {}", err),
                }
            }
            Err(e) => eprintln!("Error reading data: {}", e),
        }

        let mut res = String::new();

        print!("> ");
        io::stdout().flush()?;
        io::stdin().read_line(&mut res).expect("Failed to read line");
        let res = res.trim();

        if res.is_empty() {
            continue;
        }

        if res.starts_with("@ ") {
            let mut parts = res.splitn(2, ' ');
            let _ = parts.next().unwrap();
            let value = parts.next().unwrap();
            address = match value.parse() {
                Ok(addr) => addr,
                Err(_) => {
                    eprintln!("Invalid address.");
                    continue;
                }
            };
            continue;
        }

        if res.starts_with("# ") {
            let mut parts = res.splitn(2, ' ');
            let _ = parts.next().unwrap();
            let value = parts.next().unwrap();
            length = match value.parse() {
                Ok(len) => len,
                Err(_) => {
                    eprintln!("Invalid length.");
                    continue;
                }
            };
            continue;
        }

        let data = res.as_bytes();
        if let Err(err) = spdm.write_data(address, data) {
            eprintln!("Error writing data: {err}");
        }
    }
}
