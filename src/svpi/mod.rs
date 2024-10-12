mod utils;
use utils::*;

pub fn init_segments(memory_size: u32) -> std::io::Result<()> {
    let mut seg_mgmt = get_segment_manager()?;
    seg_mgmt.init_segments(memory_size)?;
    println!("Device initialized!");
    Ok(())
}

pub fn format_data() -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        seg_mgmt.format_data()?;
        println!("Data formatted!");
    }
    Ok(())
}

pub fn print_segments_info() -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        print_memory_state(&seg_mgmt, None);
        println!("{}", "=".repeat(109));
        println!("| {:32} | {:70} |", "Name", "Data");
        println!("{}", "=".repeat(109));
        for seg in seg_mgmt.get_segments_info().iter() {
            let data = seg_mgmt.read_segment_data(seg)?;
            println!("| {:32} | {:70} |", seg.get_name(), data);
            println!("{}", "-".repeat(109));
        }
    }
    Ok(())
}

pub fn set_segment(name: &str, data: &str) -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        if let Some(seg) = seg_mgmt.add_segment(name, data.len() as u32).map(|seg| seg.cloned())? {
            seg_mgmt.write_segment_data(&seg, &data)?;
            print_segment(&mut seg_mgmt, &seg)?;
            println!("Data saved!");
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
            let data = seg_mgmt.read_segment_data(&seg)?;
            print_segment(&mut seg_mgmt, &seg)?;
            println!("Data: {}", data);
        } else {
            println!("Data not found!");
        }
    }
    Ok(())
}

pub fn remove_segment(name: &str) -> std::io::Result<()> {
    if let Some(mut seg_mgmt) = load_segments_info()? {
        let seg = seg_mgmt.find_segment_by_name(name).cloned();
        if let Some(seg) = seg {
            print_segment(&mut seg_mgmt, &seg)?;
            seg_mgmt.remove_segment(seg.index)?;
            println!("Data removed!");
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
