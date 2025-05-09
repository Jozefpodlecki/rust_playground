mod load;
mod get_exercises;
mod get_markdown;
mod update_session;

use tauri::generate_handler;

pub fn generate_handlers() -> Box<dyn Fn(tauri::ipc::Invoke) -> bool + Send + Sync> {
    Box::new(generate_handler![
        load::load,
        get_exercises::get_exercises,
        get_markdown::get_markdown
    ])
}
