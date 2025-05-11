use chrono::NaiveDate;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadResult {
    pub app_name: String,
    pub rust_version: String,
    pub github_link: String,
    pub version: String,
    pub loaded_on: DateTime<Utc>,
}

#[derive(Debug, Default, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Exercise {
    id: Uuid,
    name: String,
    markdown: String,
    created_on: DateTime<Utc>,
}

#[derive(Debug, Default, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExerciseSession {
    pub id: Uuid,
    pub exercise_id: Uuid,
    pub folder_path: String,
    pub started_on: DateTime<Utc>,
    pub completed_on: Option<DateTime<Utc>>,
}

#[derive(Debug, Default, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyResult {
    
    
}


#[derive(Debug, Default, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateExerciseSession {
    pub exercise_id: Uuid,
    pub folder_path: Option<String>,
}

#[derive(Debug, Default, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateExerciseSession {
    pub id: Uuid,
    pub exercise_id: Uuid,
    pub folder_path: Option<String>,
    pub completed_on: Option<DateTime<Utc>>,
}
