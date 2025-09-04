pub fn get_command() -> Option<String> {
    std::env::args().nth(1)
}

pub fn get_param_by_id(id: usize) -> Option<String> {
    std::env::args().nth(id + 2)
}

pub fn get_param_by_flag(flag: &str) -> Option<String> {
    let mut iter = std::env::args().skip(2);
    while let Some(arg) = iter.next() {
        if arg == flag {
            return iter.next();
        }
    }
    None
}

pub fn check_flags(flags: &[&str]) -> bool {
    std::env::args()
        .skip(2)
        .any(|arg| flags.contains(&arg.as_str()))
}
