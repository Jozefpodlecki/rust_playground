use std::error::Error;

use tauri::{App, Manager};



pub fn setup_app(app: &mut App) -> Result<(), Box<dyn Error>> {

    let window = app.get_webview_window("main").unwrap();
    window.set_fullscreen(true)?;

    Ok(())
}