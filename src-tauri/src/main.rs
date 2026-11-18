#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::process::Command;
use serde::Serialize;
use tauri::Manager;
use std::path::PathBuf;

#[derive(Serialize)]
struct CommandResult {
    success: bool,
    output: String,
}

// دالة للبحث عن مسار adb.exe داخل التطبيق المثبت
fn get_adb_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let mut path = app_handle.path().resource_dir().unwrap_or_default();
    // المسار الصحيح لملفات الـ resources في Tauri v2
    path.push("bin");
    path.push("adb.exe");
    path
}

#[tauri::command]
fn execute_adb(app_handle: tauri::AppHandle, command: String) -> CommandResult {
    let adb_path = get_adb_path(&app_handle);
    let args: Vec<&str> = command.split_whitespace().collect();
    
    let output = Command::new(adb_path).args(&args).output();

    match output {
        Ok(out) => CommandResult {
            success: out.status.success(),
            output: String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr).to_string(),
        },
        Err(e) => CommandResult {
            success: false,
            output: format!("خطأ في تشغيل ADB. تأكد من تثبيت التعريفات: {}", e),
        },
    }
}

#[tauri::command]
fn check_device(app_handle: tauri::AppHandle) -> CommandResult {
    let adb_path = get_adb_path(&app_handle);
    
    // أمر adb devices للبحث عن الهواتف
    let output = Command::new(adb_path).arg("devices").output();

    match output {
        Ok(out) => {
            let text = String::from_utf8_lossy(&out.stdout).to_string();
            let lines: Vec<&str> = text.lines().collect();
            
            // إذا وجد سطراً ثانياً يحتوي على بيانات، فهذا يعني وجود هاتف متصل
            if lines.len() > 1 && !lines[1].trim().is_empty() {
                let serial = lines[1].split_whitespace().next().unwrap_or("").to_string();
                if serial != "" && serial != "List" {
                    return CommandResult { success: true, output: serial };
                }
            }
            CommandResult { success: false, output: "لم يتم العثور على هاتف متصل. تأكد من تفعيل USB Debugging.".to_string() }
        },
        Err(e) => CommandResult { success: false, output: format!("فشل تشغيل ADB: {}", e) },
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![execute_adb, check_device])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
