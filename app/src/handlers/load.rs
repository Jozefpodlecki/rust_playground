
use std::{process::Command, sync::Arc};
use chrono::Utc;
use tauri::{command, App, AppHandle, State};
use std::error::Error as StdError;

use crate::{error::AppError, models::LoadResult, services::AppReadyState};

#[command]
pub async fn load(
    app_ready_state: State<'_, Arc<AppReadyState>>,
    app_handle: AppHandle) -> Result<LoadResult, AppError> {
    app_ready_state.mark_ready();

    let version = app_handle.package_info().version.to_string();

    let rust_version = get_rust_version()?;

    let result = LoadResult {
        app_name: "Rust Playground".into(),
        rust_version,
        github_link: "https://github.com/Jozefpodlecki/rust_playground".into(),
        loaded_on: Utc::now(),
        version
    };

    Ok(result)
}

pub fn get_rust_version() -> Result<String, AppError> {
    let output = Command::new("rustc")
        .arg("--version")
        .output()
        .map_err(|e| AppError::Generic(Box::new(e)))?;

    if !output.status.success() {
        return Err(AppError::Generic(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "rustc returned an error",
        ))));
    }

    let rust_version = String::from_utf8(output.stdout)
        .map_err(|e| AppError::Generic(Box::new(e)))?;

    let version = rust_version
        .split_whitespace()
        .take(2)
        .collect::<Vec<&str>>()
        .join(" ");

    Ok(version.trim().to_string())
}