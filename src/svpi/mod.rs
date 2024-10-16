mod utils;
use utils::*;

use crate::{seg_mgmt::{DataType, SegmentManager}, utils::{args, console, crypto::{decrypt, encrypt}}};

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

pub fn set_root_password() -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        if seg_mgmt.is_root_password_set()? {
            println!("Root password already set!");
            if !console::confirm("Do you want to change the root password?") {
                return Ok(());
            }
        }

        let root_password = if let Some(root_password) = console::get_password(false, true, Some("Root Password".to_string())) {
            root_password
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Root password is required!"));
        };
    
        let password = console::get_password(false, true, Some("Password".to_string()));
        let password = if let Some(password) = password {
            password
        } else {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Password is required!"));
        };
    
        let encrypted_root_password = encrypt(root_password.as_bytes(), password.as_bytes())
            .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid password!"))?;
        
        seg_mgmt.set_root_password(&encrypted_root_password)?;
    }
    Ok(())
}

pub fn reset_root_password() -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        if !seg_mgmt.is_root_password_set()? {
            println!("Root password not set!");
            return Ok(());
        }
        if !console::confirm("Are you sure you want to remove the root password?") {
            return Ok(());
        }
        seg_mgmt.reset_root_password()?;
        println!("Root password removed!");
    }
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
    if let Some(mut seg_mgmt) = load_segments_info()? {
        print_memory_state(&seg_mgmt, None);
        let segs = seg_mgmt.get_segments_info();
        if !segs.is_empty() {
            let password = if segs.iter().any(|seg| seg.data_type == DataType::Encrypted) {
                get_password(&mut seg_mgmt, true, false)?
            } else {
                None
            };
            print_segments(&mut seg_mgmt, segs, password.as_ref(), PrintType::List)?;
        } else {
            println!("No data found!");
        }
    }
    Ok(())
}

pub fn set_segment(name: &str, data: &str) -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        let password = get_password(&mut seg_mgmt, true, false)?;
        let (data, data_type) = if let Some(password) = &password {
            (encrypt(data.as_bytes(), password.as_bytes())
                .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid password!"))?,
                DataType::Encrypted)
        } else {
            (data.as_bytes().to_vec(), DataType::Plain)
        };
        let seg = seg_mgmt.set_segment(name, &data, data_type).map(|seg| seg.cloned())?;
        if let Some(seg) = seg {
            print_segments(&mut seg_mgmt, vec![seg], password.as_ref(), PrintType::Set)?;
            println!("Data '{}' saved!", name);
        } else {
            println!("Not enough memory!");
            println!("Please optimize the memory: svpi optimize");
        }
    }
    Ok(())
}

pub fn get_segment(name: &str) -> std::io::Result<Option<String>> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        if let Some(seg) = seg_mgmt.find_segment_by_name(name).cloned() {
            if seg.data_type == DataType::Encrypted {
                let password = get_password(&mut seg_mgmt, true, false)?;
                if let Some(password) = password {
                    print_segments(&mut seg_mgmt, vec![seg.clone()], Some(&password), PrintType::Get)?;
                    let data = seg_mgmt.read_segment_data(&seg)?;
                    let data = decrypt(&data, password.as_bytes())
                        .map_err(|_| std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid password!"))?;
                    return Ok(Some(String::from_utf8_lossy(data.as_slice()).into_owned()));
                }
            } else {
                print_segments(&mut seg_mgmt, vec![seg.clone()], None, PrintType::Get)?;
                let data = seg_mgmt.read_segment_data(&seg)?;
                return Ok(Some(String::from_utf8_lossy(data.as_slice()).into_owned()));
            }
        } else {
            println!("Data '{}' not found!", name);
        }
    }
    Ok(None)
}

pub fn remove_segment(name: &str) -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        let seg = seg_mgmt.find_segment_by_name(name).cloned();
        if let Some(seg) = seg {
            print_segments(&mut seg_mgmt, vec![seg.clone()], None, PrintType::Remove)?;
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

pub fn export(file_name: &str) -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        let password = get_password(&mut seg_mgmt, true, false)?;
        let segs = seg_mgmt.get_segments_info();
        export_to_file(&mut seg_mgmt, segs, file_name, password.as_ref())?;
        println!("Data exported to '{}'", file_name);
    }
    Ok(())
}

pub fn import(file_name: &str) -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        let password = get_password(&mut seg_mgmt, true, false)?;
        import_from_file(&mut seg_mgmt, file_name, password.as_ref())?;
        println!("Data imported from '{}'", file_name);
    }
    Ok(())
}