mod error;
mod exercise_manager;
mod handlers;
mod models;
mod panic_hook;
mod services;
mod setup_app;

use handlers::generate_handlers;
pub use setup_app::setup_app;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    panic_hook::set_hook();

    tauri::Builder::default()
        .plugin(tauri_plugin_log::Builder::new()
            .level_for("tao::platform_impl", log::LevelFilter::Error)
            .level_for("tao", log::LevelFilter::Error)
            .level_for("sqlx::query", log::LevelFilter::Warn)
            .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::Stdout,
        ))
        .build())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(generate_handlers())
        .setup(setup_app)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
