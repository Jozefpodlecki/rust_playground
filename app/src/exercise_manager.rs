use chrono::{DateTime, Utc};
use sqlx::{Pool, Sqlite};
use anyhow::*;
use uuid::Uuid;

use crate::models::Exercise;

pub struct ExerciseManager {
    pool: Pool<Sqlite>
}

impl ExerciseManager {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn get_exercises(&self) -> Result<Vec<Exercise>> {
        
        let exercises: Vec<Exercise> = sqlx::query_as::<_, Exercise>(
        "SELECT id, name, markdown, created_on, completed_on FROM exercise"
            )
            .fetch_all(&self.pool)
            .await?;

        Ok(exercises)
    }

    pub async fn update_exercise_session(
        &self,
        exercise_id: Uuid,
        folder_path: Option<String>,
        completed_on: Option<DateTime<Utc>>,
    ) -> Result<()> {
        let mut sql = String::from("UPDATE exercise_session SET ");
        let mut sets = Vec::new();

        if folder_path.is_some() {
            sets.push("folder_path = ?");
        }
        if completed_on.is_some() {
            sets.push("completed_on = ?");
        }

        if sets.is_empty() {
            return Ok(());
        }

        sql.push_str(&sets.join(", "));
        sql.push_str(" WHERE id = ?");

        let mut query = sqlx::query(&sql);

        if let Some(fp) = folder_path {
            query = query.bind(fp);
        }
        if let Some(co) = completed_on {
            query = query.bind(co);
        }

        query = query.bind(exercise_id);
        query.execute(&self.pool).await?;

    Ok(())

    }

}