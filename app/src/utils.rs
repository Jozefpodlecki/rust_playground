use std::process::Command;

use crate::error::AppError;

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

    let rust_version =
        String::from_utf8(output.stdout).map_err(|e| AppError::Generic(Box::new(e)))?;

    let version = rust_version
        .split_whitespace()
        .take(2)
        .collect::<Vec<&str>>()
        .join(" ");

    Ok(version.trim().to_string())
}
