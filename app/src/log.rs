
pub fn logInfo(func: &str, s: String) {
    println!("INFO : [{}] -> {}", func, s)
}

pub fn logWarn(func: &str, s: String) {
    println!("WARN : [{}] -> {}", func, s)
}

pub fn logError(func: &str, s: String) {
    println!("ERROR: [{}] -> {}", func, s)
}

pub fn logDebug(func: &str, s: String) {
    println!("DEBUG: [{}] -> {}", func, s)
}