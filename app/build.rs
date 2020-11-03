fn main() {
    println!("cargo:rustc-cdylib-link-arg=-undefined");
    if cfg!(target_os = "macos") {
        println!("cargo:rustc-cdylib-link-arg=dynamic_lookup");
    }
}
