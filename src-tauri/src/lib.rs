#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_process::init())
        .invoke_handler(tauri::generate_handler![read_text_file, write_text_file, list_directory])
        .run(tauri::generate_context!())
        .expect("error while running Pezli Editor");
}

#[tauri::command]
fn read_text_file(path: String) -> Result<String, String> {
    std::fs::read_to_string(&path).map_err(|error| error.to_string())
}

#[tauri::command]
fn write_text_file(path: String, content: String) -> Result<(), String> {
    std::fs::write(&path, content).map_err(|error| error.to_string())
}

#[derive(serde::Serialize)]
struct DirectoryEntry {
    name: String,
    path: String,
    is_dir: bool,
}

#[tauri::command]
fn list_directory(path: String) -> Result<Vec<DirectoryEntry>, String> {
    let mut entries = Vec::new();
    let directory = std::fs::read_dir(&path).map_err(|error| error.to_string())?;
    for entry in directory {
        let entry = entry.map_err(|error| error.to_string())?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_dir = entry
            .file_type()
            .map_err(|error| error.to_string())?
            .is_dir();
        let full_path = entry.path().to_string_lossy().into_owned();
        entries.push(DirectoryEntry {
            name,
            path: full_path,
            is_dir,
        });
    }
    entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
        (true, false) => std::cmp::Ordering::Less,
        (false, true) => std::cmp::Ordering::Greater,
        _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
    });
    Ok(entries)
}
