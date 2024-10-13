use std::io::Write;

use crate::{seg_mgmt::{DataType, Segment, SegmentManager}, spdm::SerialPortDataManager, utils::crypto::{decrypt, encrypt}};


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

pub fn print_segments(seg_mgmt: &mut SegmentManager, seg: Vec<Segment>, password: Option<&String>) -> std::io::Result<()> {
    println!("{}", "=".repeat(112));
    println!("| {:32} | {:20} | {:50} |", "Name", "Data Type", "Data");
    println!("{}", "=".repeat(112));
    for seg in seg.iter() {
        let data = seg_mgmt.read_segment_data(seg)?;
        let (data_type, data) = if seg.data_type == DataType::Encrypted {
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
            let data = seg_mgmt.read_segment_data(seg)?;
            let data = String::from_utf8_lossy(data.as_slice()).into_owned();
            ("Plain", data)
        };
        println!("| {:32} | {:20} | {:50} |", seg.get_name(), data_type, data);
        println!("{}", "-".repeat(112));
    }
    Ok(())
}

pub fn export_to_file(seg_mgmt: &mut SegmentManager, seg: Vec<Segment>, file_path: &str, password: Option<&String>) -> std::io::Result<()> {
    let mut list = Vec::new();
    for seg in seg.iter() {
        let data = seg_mgmt.read_segment_data(seg)?;
        let data = if seg.data_type == DataType::Encrypted {
            if let Some(password) = password {
                match decrypt(&data, password.as_bytes()) {
                    Ok(data) => String::from_utf8_lossy(data.as_slice()).into_owned(),
                    Err(_) => "Password does not match".to_string(),
                }
            } else {
                format!("{} bytes", seg.size)
            }
        } else {
            String::from_utf8_lossy(data.as_slice()).into_owned()
        };
        list.push((seg.get_name(), data));
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
        (name, data)
    }).collect::<Vec<(&str, &str)>>();
    let data_type = if password.is_some() {
        DataType::Encrypted
    } else {
        DataType::Plain
    };
    for (name, data) in list.iter() {
        let data = if let Some(password) = password {
            let data = encrypt(data.as_bytes(), password.as_bytes())
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid password!"))?;
            data
        } else {
            data.as_bytes().to_vec()
        };
        let seg = seg_mgmt.add_segment(name, data.len() as u32, data_type.clone()).map(|seg| seg.cloned())?;
        if let Some(seg) = seg {
            seg_mgmt.write_segment_data(&seg, &data)?;
        } else {
            println!("Not enough memory!");
            println!("Please optimize the memory: svpi optimize");
            break;
        }
    }
    Ok(())
}