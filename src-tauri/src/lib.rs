// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/

#[tauri::command]
fn quit_app() {
    println!("quit_app command invoked"); // Debug log
    std::process::exit(0); // Exits the app
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![quit_app])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
