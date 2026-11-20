use tauri::Manager;
use std::process::Command;
use std::path::PathBuf;

#[derive(serde::Serialize)]
struct CommandResult {
    success: bool,
    output: String,
}

#[tauri::command]
fn check_device(app_handle: tauri::AppHandle) -> CommandResult {
    let adb = get_adb_path(&app_handle);
    let out = Command::new(&adb).arg("devices").output();
    match out {
        Ok(o) => {
            let txt = String::from_utf8_lossy(&o.stdout).to_string();
            let lines: Vec<&str> = txt.lines().skip(1).filter(|l| !l.trim().is_empty()).collect();
            if lines.is_empty() {
                return CommandResult { success: false, output: "لم يتم العثور على جهاز - تأكد من تفعيل تصحيح USB ووصل الكابل".to_string() };
            }
            for line in &lines {
                if line.contains("device") && !line.contains("unauthorized") {
                    let serial = line.split_whitespace().next().unwrap_or("").to_string();
                    return CommandResult { success: true, output: serial };
                }
                if line.contains("unauthorized") {
                    return CommandResult { success: false, output: "الجهاز غير مصرح - اضغط 'السماح' على شاشة الهاتف".to_string() };
                }
            }
            CommandResult { success: false, output: "جهاز غير متصل بشكل صحيح".to_string() }
        }
        Err(e) => CommandResult { success: false, output: format!("ADB غير موجود: {}", e) },
    }
}

#[tauri::command]
fn execute_adb(app_handle: tauri::AppHandle, command: String) -> CommandResult {
    let adb = get_adb_path(&app_handle);
    let parts: Vec<&str> = command.split_whitespace().collect();
    let out = Command::new(&adb).args(&parts).output();
    match out {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).to_string();
            let stderr = String::from_utf8_lossy(&o.stderr).to_string();
            let combined = format!("{}{}", stdout, stderr);
            CommandResult { success: o.status.success(), output: if combined.trim().is_empty() { "تم التنفيذ بنجاح".to_string() } else { combined } }
        }
        Err(e) => CommandResult { success: false, output: format!("خطأ: {}", e) },
    }
}

#[tauri::command]
fn install_drivers(app_handle: tauri::AppHandle) -> CommandResult {
    // الإصلاح هنا: نجيب المسار بطريقة ما تسبب خطأ join
    let resource_dir: PathBuf = app_handle
        .path()
        .resource_dir()
        .unwrap_or_else(|_| PathBuf::from("."));

    let drivers_dir: PathBuf = resource_dir.join("drivers");

    let mut found = String::new();
    if let Ok(entries) = std::fs::read_dir(&drivers_dir) {
        for e in entries.flatten() {
            found.push_str(&format!("- {}\n", e.file_name().to_string_lossy()));
        }
    }
    if found.is_empty() {
        return CommandResult { success: false, output: format!("مجلد drivers فاضي! المسار: {:?}", drivers_dir) };
    }

    let names = vec!["Universal ADB Driver v5.0.exe", "Universal_ADB_Driver.exe", "adb-driver-installer.exe"];
    for name in names {
        let driver_path: PathBuf = drivers_dir.join(name);
        if driver_path.exists() {
            let ps = format!("Start-Process -FilePath '{}' -ArgumentList '/S' -Verb RunAs", driver_path.display());
            match Command::new("powershell").args(["-Command", &ps]).spawn() {
                Ok(_) => return CommandResult { success: true, output: format!("تم تشغيل المثبت كمسؤول: {} - اضغط Yes إذا طلعت نافذة ويندوز", name) },
                Err(e) => return CommandResult { success: false, output: format!("فشل التشغيل: {}", e) },
            }
        }
    }
    CommandResult { success: false, output: format!("لم يتم العثور على ملف التعريف. الموجود:\n{}", found) }
}

fn get_adb_path(app_handle: &tauri::AppHandle) -> PathBuf {
    let resource_dir: PathBuf = app_handle
        .path()
        .resource_dir()
        .unwrap_or_else(|_| PathBuf::from("."));
    let bundled: PathBuf = resource_dir.join("bin").join("adb.exe");
    if bundled.exists() {
        bundled
    } else {
        PathBuf::from("adb")
    }
}

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![check_device, execute_adb, install_drivers])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
