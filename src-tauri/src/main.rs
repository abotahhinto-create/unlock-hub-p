#[tauri::command]
fn install_drivers(app_handle: tauri::AppHandle) -> CommandResult {
    let resource_dir = match app_handle.path().resource_dir() {
        Ok(p) => p,
        Err(_) => return CommandResult { success: false, output: "لم يتم العثور على مجلد الريسورس".to_string() },
    };

    // طباعة المسار للتشخيص في السجل
    let drivers_dir = resource_dir.join("drivers");
    let mut found_files = String::new();
    if let Ok(entries) = std::fs::read_dir(&drivers_dir) {
        for e in entries.flatten() {
            found_files.push_str(&format!("- {}\n", e.file_name().to_string_lossy()));
        }
    }
    if found_files.is_empty() {
        return CommandResult { success: false, output: format!("مجلد drivers فاضي! المسار: {:?} - تأكد أنك رفعت ملف التعريف في GitHub داخل src-tauri/drivers/", drivers_dir) };
    }

    let possible_names = vec![
        "Universal ADB Driver v5.0.exe",
        "Universal_ADB_Driver.exe",
        "adb-driver-installer.exe",
    ];

    for name in possible_names {
        let driver_path = drivers_dir.join(name);
        if driver_path.exists() {
            // تشغيل بصلاحية مسؤول
            let ps_cmd = format!("Start-Process -FilePath '{}' -ArgumentList '/S' -Verb RunAs", driver_path.display());
            match Command::new("powershell").args(["-Command", &ps_cmd]).spawn() {
                Ok(_) => return CommandResult { success: true, output: format!("تم تشغيل المثبت كمسؤول: {} - إذا طلعت نافذة ويندوز اضغط Yes وانتظر دقيقة", name) },
                Err(e) => return CommandResult { success: false, output: format!("فشل التشغيل كمسؤول: {}", e) },
            }
        }
    }

    CommandResult { success: false, output: format!("لم يتم العثور على ملف التعريف. الملفات الموجودة:\n{} \nأعد تسمية الملف لـ Universal ADB Driver v5.0.exe", found_files) }
}
