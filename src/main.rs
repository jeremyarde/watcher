use std::process::Command;

use crate::scripts::{ScriptType, load_scripts};

mod scripts;

fn main() {
    let scripts = load_scripts();

    let script_output = Command::new("osascript")
        .arg("-e")
        .arg(scripts.get(&ScriptType::ForegroundApp).unwrap())
        .output()
        .expect("Failed to execute script");

    println!("{}", String::from_utf8_lossy(&script_output.stdout));
}
