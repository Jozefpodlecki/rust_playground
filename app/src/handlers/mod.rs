mod exercise;
mod get_markdown;
mod load;
mod session;

use tauri::generate_handler;

pub fn generate_handlers() -> Box<dyn Fn(tauri::ipc::Invoke) -> bool + Send + Sync> {
    Box::new(generate_handler![
        load::load,
        exercise::get_exercises,
        exercise::verify_exercise,
        get_markdown::get_markdown,
        exercise::get_last_exercise_session,
        session::update_session,
        session::create_session
    ])
}
