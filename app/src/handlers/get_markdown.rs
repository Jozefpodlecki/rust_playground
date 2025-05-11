use std::{path::PathBuf, sync::Arc};
use log::info;
use tauri::{State, command};
use tokio::fs;

use crate::{error::AppError, exercise_manager::ExerciseManager};

#[command]
pub async fn get_markdown(
    exercise_manager: State<'_, Arc<ExerciseManager>>,
    markdown_name: String
) -> Result<String, AppError> {

    let mut path = PathBuf::from("../exercises");
    path.push(&markdown_name);

    info!("{} {:?}", markdown_name, path);

    if !path.exists() {
        return Err(AppError::Unknown);
    }

    let content = fs::read_to_string(&path)
        .await
        .map_err(|err| AppError::Unknown)?;

    Ok(content)
}
