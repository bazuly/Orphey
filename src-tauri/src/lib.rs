mod commands;

use commands::AppState;
use orphey::backend::Backend;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let backend = Backend::open().expect("failed to open database");

    tauri::Builder::default()
        .manage(AppState(std::sync::Mutex::new(backend)))
        .invoke_handler(tauri::generate_handler![
            commands::bootstrap,
            commands::list_requests,
            commands::create_room,
            commands::delete_room,
            commands::set_room_base_url,
            commands::delete_request,
            commands::send_request,
            commands::run_load,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}