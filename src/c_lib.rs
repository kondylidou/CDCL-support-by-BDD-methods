#![crate_name = "c_library"]

pub extern "C" fn init() {
    println!("Hello from Rust!");
}