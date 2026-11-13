#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::process::Command;
use serde::Serialize;

#[derive(Serialize)]
struct Result { success: bool, output: String }

#[tauri::command]
fn execute_adb(command: String) -> Result {
    let args: Vec<&str> = command.split_whitespace().collect();
    let output = Command::new("adb").args(&args).output();
    match output {
        Ok(out) => Result {
            success: out.status.success(),
            output: String::from_utf8_lossy(&out.stdout).to_string(),
        },
        Err(e) => Result { success: false, output: format!("ADB Error: {}", e) },
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![execute_adb])
        .run(tauri::generate_context!())
        .expect("error");
}
