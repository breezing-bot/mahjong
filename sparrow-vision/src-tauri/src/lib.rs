mod commands;
mod scoring;
mod tile_vocab;
mod types;
mod vision;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            commands::recognize_image,
            commands::analyze_hand
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
