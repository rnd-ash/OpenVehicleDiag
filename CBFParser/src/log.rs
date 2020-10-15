

pub struct Logger{} 

impl Logger {
    pub fn logFatal3(ecx: &str, a2: &str, a3: &str, a4: u32) {
        println!("{} {} {} {}", ecx, a2, a3, a4);
    }
    pub fn logFatal2(ecx: &str, a2: &str, a3: u32) {
        println!("{} {} {}", ecx, a2, a3);
    }
}