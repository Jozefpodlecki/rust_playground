use std::sync::Arc;

use tauri::{State, command};

use crate::{error::AppError, exercise_manager::ExerciseManager, models::UpdateExerciseSession};

#[command]
pub async fn update_session(
    exercise_manager: State<'_, Arc<ExerciseManager>>,
    payload: UpdateExerciseSession,
) -> Result<(), AppError> {
    exercise_manager
        .update_exercise_session(
            payload.exercise_id,
            payload.folder_path,
            payload.completed_on,
        )
        .await
        .map_err(|e| AppError::Unknown)?;

    Ok(())
}
