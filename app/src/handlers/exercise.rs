use std::{process::Command, sync::Arc};
use tauri::{State, command};
use uuid::Uuid;

use crate::{
    error::AppError,
    exercise_manager::ExerciseManager,
    models::{Exercise, ExerciseSession, VerifyResult},
};

#[command]
pub async fn verify_exercise(
    exercise_manager: State<'_, Arc<ExerciseManager>>,
    session_id: Uuid
) -> Result<VerifyResult, AppError> {
    
    let session = exercise_manager.get_session_by_id(session_id)
        .await
        .map_err(|err| AppError::Sqlite(err.to_string()))?
        .unwrap();

    let status = Command::new("cargo")
        .arg("run")
        .current_dir(session.folder_path)
        .status()
        .map_err(|err| AppError::Command(err.to_string()))?;

    let result = VerifyResult {
        
    };

    Ok(result)
}

#[command]
pub async fn get_exercises(
    exercise_manager: State<'_, Arc<ExerciseManager>>,
) -> Result<Vec<Exercise>, AppError> {
    let exercises = exercise_manager
        .get_exercises()
        .await
        .map_err(|err| AppError::Sqlite(err.to_string()))?;

    Ok(exercises)
}

#[command]
pub async fn get_last_exercise_session(
    exercise_manager: State<'_, Arc<ExerciseManager>>,
) -> Result<Option<ExerciseSession>, AppError> {
    let exercise_session = exercise_manager
        .get_last_exercise_session()
        .await
        .map_err(|err| AppError::Sqlite(err.to_string()))?;

    Ok(exercise_session)
}
