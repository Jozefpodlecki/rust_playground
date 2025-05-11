use std::sync::Arc;

use tauri::{State, command};
use uuid::Uuid;

use crate::{
    error::AppError,
    exercise_manager::ExerciseManager,
    models::{CreateExerciseSession, ExerciseSession, UpdateExerciseSession},
};

#[command]
pub async fn create_session(
    exercise_manager: State<'_, Arc<ExerciseManager>>,
    payload: CreateExerciseSession,
) -> Result<ExerciseSession, AppError> {
    let session = exercise_manager
        .create_exercise_session(payload)
        .await
        .map_err(|e| AppError::Unknown)?;

    Ok(session)
}

#[command]
pub async fn update_session(
    exercise_manager: State<'_, Arc<ExerciseManager>>,
    payload: UpdateExerciseSession,
) -> Result<(), AppError> {
    exercise_manager
        .update_exercise_session(payload)
        .await
        .map_err(|e| AppError::Unknown)?;

    Ok(())
}
