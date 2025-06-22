mod utils;
pub use utils::*;
pub mod result;

use self::result::{Error, Result};
use crate::{
    seg_mgmt::{Data, SegmentError},
    utils::{args, terminal},
};

pub fn init_segments(memory_size: u32) -> Result<()> {
    let mut seg_mgmt = get_segment_manager()?;
    if seg_mgmt.check_init_data().map_err(Into::into)? {
        println!("Device already initialized!");
    }
    if !terminal::confirm("Are you sure you want to initialize the device?") {
        return Ok(());
    }
    seg_mgmt.init_segments(memory_size).map_err(Into::into)?;
    println!("Device initialized!");
    Ok(())
}

pub fn set_root_password() -> Result<()> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        if seg_mgmt.is_root_password_set().map_err(Into::into)? {
            println!("Root password already set!");
            if !terminal::confirm("Do you want to change the root password?") {
                return Ok(());
            }
        }

        let root_password = if let Some(root_password) =
            terminal::get_password(false, true, Some("Root Password".to_string()))
        {
            root_password
        } else {
            return Err(Error::PasswordIsRequired);
        };

        let password = terminal::get_password(false, true, Some("Password".to_string()));
        let password = if let Some(password) = password {
            password
        } else {
            return Err(Error::PasswordIsRequired);
        };

        seg_mgmt.set_root_password(&root_password, &password)?;
    }
    Ok(())
}

pub fn reset_root_password() -> Result<()> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        if !seg_mgmt.is_root_password_set().map_err(Into::into)? {
            println!("Root password not set!");
            return Ok(());
        }
        if !terminal::confirm("Are you sure you want to remove the root password?") {
            return Ok(());
        }
        seg_mgmt.reset_root_password().map_err(Into::into)?;
        println!("Root password removed!");
    }
    Ok(())
}

pub fn get_root_password() -> Result<Option<String>> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        if seg_mgmt.is_root_password_set().map_err(Into::into)? {
            let password = terminal::get_password(false, true, Some("Password".to_string()));
            let password = if let Some(password) = password {
                password
            } else {
                return Err(Error::PasswordIsRequired);
            };
            let root_password = seg_mgmt.get_root_password(&password, None)?;
            if !args::check_flag(vec!["--clipboard", "-c"]) {
                println!("Root password: {}", root_password);
            }
            return Ok(Some(root_password));
        } else {
            println!("Root password not set!");
        }
    }
    Ok(None)
}

pub fn format_data() -> Result<()> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        if !terminal::confirm("Are you sure you want to format the data?") {
            return Ok(());
        }
        seg_mgmt.format_data().map_err(Into::into)?;
        println!("Data formatted!");
    }
    Ok(())
}

pub fn print_segments_meta() -> Result<()> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        print_memory_state(&seg_mgmt, None);
        let segs = seg_mgmt.get_active_segments();
        if !segs.is_empty() {
            let password = if segs.iter().any(|seg| seg.is_encrypted) {
                get_password(&mut seg_mgmt, true, false)?
            } else {
                None
            };
            let segs = seg_mgmt.get_active_segments_mut();
            print_segments(segs, password.as_deref(), true)?;
        } else {
            println!("No data found!");
        }
    }
    Ok(())
}

pub fn set_segment(name: &str, data: &str) -> Result<()> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        let password = get_password(&mut seg_mgmt, true, false)?;
        let data = Data::detect_type(data);
        let is_created = seg_mgmt
            .set_segment(name, data, password.as_deref())
            .map_err(Into::into)?
            .is_some();
        if is_created {
            let seg = seg_mgmt
                .find_segment_by_name(name)
                .ok_or(Error::SegmentError(SegmentError::NotFound))?;
            print_segments(vec![seg], password.as_deref(), false)?;
            println!("Data '{}' saved!", name);
        } else {
            println!("Not enough memory!");
            println!("Please optimize the memory: svpi optimize");
        }
    }
    Ok(())
}

pub fn get_segment(name: &str) -> Result<Option<String>> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        let is_encrypted = if let Some(seg) = seg_mgmt.find_segment_by_name(name) {
            seg.is_encrypted
        } else {
            println!("Data '{}' not found!", name);
            return Ok(None);
        };
        let password = get_password(&mut seg_mgmt, !is_encrypted, false)?;
        if let Some(seg) = seg_mgmt.find_segment_by_name(name) {
            print_segments(vec![seg], password.as_deref(), false)?;
        }
    }
    Ok(None)
}

pub fn remove_segment(name: &str) -> Result<()> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        if let Some(seg) = seg_mgmt.find_segment_by_name(name) {
            print_segments(vec![seg], None, true)?;
            if !terminal::confirm(&format!("Are you sure you want to remove '{}'?", name)) {
                return Ok(());
            }
            seg.remove().map_err(Into::into)?;
            println!("Data '{}' removed!", name);
        } else {
            println!("Data not found!");
        }
    }
    Ok(())
}

pub fn optimize() -> Result<()> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        let optimized_size = seg_mgmt.optimize_segments().map_err(Into::into)?;
        print_memory_state(&seg_mgmt, Some(optimized_size));
        if optimized_size > 0 {
            println!("Memory optimized!");
        } else {
            println!("Memory already optimized!");
        }
    }
    Ok(())
}

pub fn export(file_name: &str) -> Result<()> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        let password = get_password(&mut seg_mgmt, true, false)?;
        export_to_file(&mut seg_mgmt, file_name, password.as_deref())?;
        println!("Data exported to '{}'", file_name);
    }
    Ok(())
}

pub fn import(file_name: &str) -> Result<()> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        let password = get_password(&mut seg_mgmt, true, false)?;
        import_from_file(&mut seg_mgmt, file_name, password.as_deref())?;
        println!("Data imported from '{}'", file_name);
    }
    Ok(())
}

pub fn save_dump(file_name: &str) -> Result<()> {
    if let Some(mut seg_mgmt) = get_inited_manager()? {
        save_dump_to_file(&mut seg_mgmt, file_name)?;
        println!("Dump saved to '{}'", file_name);
    }
    Ok(())
}

pub fn load_dump(file_name: &str) -> Result<()> {
    let mut seg_mgmt = get_segment_manager()?;
    if seg_mgmt.check_init_data().map_err(Into::into)? {
        println!("Device already initialized!");
    }
    if !terminal::confirm("Are you sure you want to load the dump?") {
        return Ok(());
    }
    load_dump_from_file(&mut seg_mgmt, file_name)?;
    println!("Dump loaded!");
    Ok(())
}
