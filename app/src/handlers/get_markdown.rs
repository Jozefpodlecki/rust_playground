
use std::{path::PathBuf, sync::Arc};
use tauri::{command, State};
use tokio::fs;

use crate::{error::AppError, exercise_manager::ExerciseManager};

#[command]
pub async fn get_markdown(
    exercise_manager: State<'_, Arc<ExerciseManager>>, id: u64) -> Result<String, AppError> {
    let mut path = PathBuf::from("exercises");
    path.push(format!("{}_exercise.md", id));

    if !path.exists() {
        return Err(AppError::Unknown);
    }

    let content = fs::read_to_string(&path)
        .await
        .map_err(|err| AppError::Unknown)?;

    Ok(content)
}