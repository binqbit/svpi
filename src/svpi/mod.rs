mod utils;
use utils::*;

use crate::{seg_mgmt::DataType, utils::{console, crypto::encrypt}};

pub fn init_segments(memory_size: u32) -> std::io::Result<()> {
    let mut seg_mgmt = get_segment_manager()?;
    if seg_mgmt.get_memory_size()?.is_some() {
        println!("Device already initialized!");
    }
    if !console::confirm("Are you sure you want to initialize the device?") {
        return Ok(());
    }
    seg_mgmt.init_segments(memory_size)?;
    println!("Device initialized!");
    Ok(())
}

pub fn format_data() -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        if !console::confirm("Are you sure you want to format the data?") {
            return Ok(());
        }
        seg_mgmt.format_data()?;
        println!("Data formatted!");
    }
    Ok(())
}

pub fn print_segments_info() -> std::io::Result<()> {
    let password = console::get_password(true);
    if let Some(mut seg_mgmt) = load_segments_info()? {
        print_memory_state(&seg_mgmt, None);
        let segs = seg_mgmt.get_segments_info();
        print_segments(&mut seg_mgmt, segs, password.as_ref())?;
    }
    Ok(())
}

pub fn set_segment(name: &str, data: &str) -> std::io::Result<()> {
    let password = console::get_password(true);
    let (data, data_type) = if let Some(password) = &password {
        (encrypt(data.as_bytes(), password.as_bytes())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid password!"))?,
            DataType::Encrypted)
    } else {
        (data.as_bytes().to_vec(), DataType::Plain)
    };
    if let Some(mut seg_mgmt) = load_segments_info()? {
        if let Some(seg) = seg_mgmt.add_segment(name, data.len() as u32, data_type).map(|seg| seg.cloned())? {
            seg_mgmt.write_segment_data(&seg, &data)?;
            print_segments(&mut seg_mgmt, vec![seg], password.as_ref())?;
            println!("Data '{}' saved!", name);
        } else {
            println!("Not enough memory!");
            println!("Please optimize the memory: svpi optimize");
        }
    }
    Ok(())
}

pub fn get_segment(name: &str) -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        if let Some(seg) = seg_mgmt.find_segment_by_name(name).cloned() {
            print_segments(&mut seg_mgmt, vec![seg.clone()], None)?;
            if seg.data_type == DataType::Encrypted {
                let password = console::get_password(false);
                if let Some(password) = password {
                    print_segments(&mut seg_mgmt, vec![seg], Some(&password))?;
                }
            }
        } else {
            println!("Data '{}' not found!", name);
        }
    }
    Ok(())
}

pub fn remove_segment(name: &str) -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        let seg = seg_mgmt.find_segment_by_name(name).cloned();
        if let Some(seg) = seg {
            print_segments(&mut seg_mgmt, vec![seg.clone()], None)?;
            if !console::confirm(&format!("Are you sure you want to remove '{}'?", name)) {
                return Ok(());
            }
            seg_mgmt.remove_segment(seg.index)?;
            println!("Data '{}' removed!", name);
        } else {
            println!("Data not found!");
        }
    }
    Ok(())
}

pub fn optimize() -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        let optimized_size = seg_mgmt.optimizate_segments()?;
        print_memory_state(&seg_mgmt, Some(optimized_size));
        if optimized_size > 0 {
            println!("Memory optimized!");
        } else {
            println!("Memory already optimized!");
        }
    }
    Ok(())
}
