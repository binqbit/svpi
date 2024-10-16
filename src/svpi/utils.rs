use std::io::Write;

use crate::{seg_mgmt::{DataType, Segment, SegmentManager}, spdm::SerialPortDataManager, utils::{args, crypto::{decrypt, encrypt}}};


pub fn load_segments_info() -> std::io::Result<Option<SegmentManager>> {
    let spdm = SerialPortDataManager::find_device();
    let mut seg_mgmt = spdm.into_segment_manager();
    if seg_mgmt.load_segments()? {
        Ok(Some(seg_mgmt))
    } else {
        println!("Device not initialized!");
        println!("Please run the init command: svpi init <memory_size>");
        Ok(None)
    }
}

pub fn get_segment_manager() -> std::io::Result<SegmentManager> {
    let spdm = SerialPortDataManager::find_device();
    Ok(spdm.into_segment_manager())
}

pub fn print_memory_state(seg_mgmt: &SegmentManager, optimized_size: Option<u32>) {
    println!("{}", "=".repeat(36));
    println!("| {:14} | {:15} |", "Memory Size", "Value (bytes)");
    println!("{}", "=".repeat(36));
    println!("| {:14} | {:15} |", "Total", seg_mgmt.memory_size);
    println!("{}", "-".repeat(36));
    println!("| {:14} | {:15} |", "Free", seg_mgmt.free_memory_size());
    println!("{}", "-".repeat(36));
    if let Some(optimized_size) = optimized_size {
        println!("| {:14} | {:15} |", "Optimized", optimized_size);
        println!("{}", "-".repeat(36));
    }
}

pub enum PrintType {
    List,
    Get,
    Set,
    Remove,
}

pub fn print_segments(seg_mgmt: &mut SegmentManager, seg: Vec<Segment>, password: Option<&String>, print_type: PrintType) -> std::io::Result<()> {
    let is_view = args::get_flag(vec!["--view", "-v"]).is_some() ||
        match print_type {
            PrintType::List => false,
            PrintType::Get => args::get_flag(vec!["--clipboard", "-c"]).is_none(),
            PrintType::Set => false,
            PrintType::Remove => false,
        };
    println!("{}", "=".repeat(112));
    println!("| {:32} | {:20} | {:50} |", "Name", "Data Type", "Data");
    println!("{}", "=".repeat(112));
    for seg in seg.iter() {
        let (data_type, data) = if seg.data_type == DataType::Encrypted {
            let data = seg_mgmt.read_segment_data(seg)?;
            if let Some(password) = password {
                match decrypt(&data, password.as_bytes()) {
                    Ok(data) => {
                        let data = String::from_utf8_lossy(data.as_slice()).into_owned();
                        ("Decrypted", data)
                    }
                    Err(_) => ("Error", "Password does not match".to_string()),
                }
            } else {
                ("Encrypted", format!("{} bytes", seg.size))
            }
        } else {
            if is_view {
                let data = seg_mgmt.read_segment_data(seg)?;
                let data = String::from_utf8_lossy(data.as_slice()).into_owned();
                ("Plain", data)
            } else {
                ("Plain", format!("{} bytes", seg.size))
            }
        };
        println!("| {:32} | {:20} | {:50} |", seg.get_name(), data_type, data);
        println!("{}", "-".repeat(112));
    }
    Ok(())
}

fn export_segment_with_type(seg: &Segment, data: &[u8], is_encrypted: bool) -> (String, String) {
    let (name, data) = if is_encrypted {
        (format!("@{}", seg.get_name()), base64::encode(&data))
    } else {
        (seg.get_name(), String::from_utf8_lossy(data).into_owned())
    };
    (name, data)
}

pub fn export_to_file(seg_mgmt: &mut SegmentManager, seg: Vec<Segment>, file_path: &str, password: Option<&String>) -> std::io::Result<()> {
    let mut list = Vec::new();
    for seg in seg.iter() {
        let data = seg_mgmt.read_segment_data(seg)?;
        let (name, data) = if seg.data_type == DataType::Encrypted {
            if let Some(password) = password {
                match decrypt(&data, password.as_bytes()) {
                    Ok(data) => export_segment_with_type(seg, &data, false),
                    Err(_) => export_segment_with_type(seg, &data, true),
                }
            } else {
                export_segment_with_type(seg, &data, true)
            }
        } else {
            export_segment_with_type(seg, &data, false)
        };
        list.push((name, data));
    }
    let text = list.iter().map(|(k, v)| format!("{}: {}\n", k, v)).collect::<String>();
    let mut file = std::fs::File::create(file_path)?;
    file.write_all(text.as_bytes())?;
    Ok(())
}

pub fn import_from_file(seg_mgmt: &mut SegmentManager, file_path: &str, password: Option<&String>) -> std::io::Result<()> {
    let text = std::fs::read_to_string(file_path)?;
    let list = text.lines().map(|line| {
        let mut parts = line.splitn(2, ": ");
        let name = parts.next().unwrap().trim();
        let data = parts.next().unwrap().trim();
        let (name, data_type) = if name.starts_with("@") {
            (&name[1..], DataType::Encrypted)
        } else {
            (name, DataType::Plain)
        };
        (name, data, data_type)
    }).collect::<Vec<(&str, &str, DataType)>>();
    for (name, data, data_type) in list.iter() {
        let (data, data_type) = if data_type == &DataType::Plain {
            if let Some(password) = password {
                let data = encrypt(data.as_bytes(), password.as_bytes())
                    .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid password!"))?;
                (data, DataType::Encrypted)
            } else {
                (data.as_bytes().to_vec(), data_type.clone())
            }
        } else {
            let data = base64::decode(data).map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid base64 data!"))?;
            (data, data_type.clone())
        };
        let seg = seg_mgmt.set_segment(name, &data, data_type.clone())?;
        if seg.is_none() {
            println!("Not enough memory!");
            println!("Please optimize the memory: svpi optimize");
            break;
        }
    }
    Ok(())
}