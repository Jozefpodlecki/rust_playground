
use std::{process::Command, sync::Arc};
use chrono::Utc;
use tauri::{command, App, AppHandle, State};
use std::error::Error as StdError;

use crate::{error::AppError, models::{Exercise, LoadResult}, services::AppReadyState};

#[command]
pub async fn update_session() -> Result<Vec<Exercise>, AppError> {

    Ok(vec![])
}