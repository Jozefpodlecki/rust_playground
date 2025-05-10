
use std::sync::Arc;
use tauri::{command, State};

use crate::{error::AppError, exercise_manager::ExerciseManager, models::Exercise};

#[command]
pub async fn get_exercises(exercise_manager: State<'_, Arc<ExerciseManager>>) -> Result<Vec<Exercise>, AppError> {

    let exercises = exercise_manager
        .get_exercises()
        .await
        .map_err(|e| AppError::Unknown)?;

    Ok(exercises)
}