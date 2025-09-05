pub fn get_command() -> Option<String> {
    for arg in std::env::args().skip(1) {
        if !arg.starts_with('-') {
            return Some(arg);
        }
    }
    None
}

pub fn get_param_by_id(id: usize) -> Option<String> {
    let mut id = id + 1;
    for arg in std::env::args().skip(1) {
        if !arg.starts_with('-') {
            if id == 0 {
                return Some(arg);
            }
            id -= 1;
        }
    }
    None
}

pub fn get_param_by_flag(flag: &str) -> Option<String> {
    let flag = format!("{}=", flag);
    let mut iter = std::env::args().skip(1);
    while let Some(arg) = iter.next() {
        if arg.starts_with(&flag) {
            let value = arg[flag.len()..].to_string();
            return Some(value);
        }
    }
    None
}

pub fn check_flags(flags: &[&str]) -> bool {
    std::env::args()
        .skip(1)
        .any(|arg| flags.iter().any(|flag| arg.starts_with(flag)))
}
