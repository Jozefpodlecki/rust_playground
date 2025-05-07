mod setup_app;
mod handlers;
mod panic_hook;
mod services;
mod models;
mod error;

use handlers::generate_handlers;
pub use setup_app::setup_app;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    panic_hook::set_hook();
    
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(generate_handlers())
        .setup(setup_app)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
