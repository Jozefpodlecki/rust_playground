use anyhow::*;
use chrono::{DateTime, Utc};
use sqlx::{Pool, Sqlite};
use uuid::Uuid;

use crate::models::{CreateExerciseSession, Exercise, ExerciseSession, UpdateExerciseSession};

pub struct ExerciseManager {
    pool: Pool<Sqlite>,
}

impl ExerciseManager {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub async fn get_exercises(&self) -> Result<Vec<Exercise>> {
        let exercises: Vec<Exercise> =
            sqlx::query_as::<_, Exercise>("
            SELECT
                id, name, markdown, created_on
            FROM exercise")
                .fetch_all(&self.pool)
                .await?;

        Ok(exercises)
    }

    pub async fn get_session_by_id(&self, session_id: Uuid) -> Result<Option<ExerciseSession>> {
        let session: Option<ExerciseSession> = sqlx::query_as::<_, ExerciseSession>(
            "SELECT
            id, exercise_id, folder_path, started_on, completed_on 
            FROM exercise_session 
            WHERE id = ?",
        )
        .bind(session_id) 
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn get_last_exercise_session(&self) -> Result<Option<ExerciseSession>> {
        let session: Option<ExerciseSession> = sqlx::query_as::<_, ExerciseSession>(
            "SELECT
            id, exercise_id, folder_path, started_on, completed_on 
            FROM exercise_session 
            ORDER BY started_on DESC 
            LIMIT 1",
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(session)
    }

    pub async fn create_exercise_session(&self, payload: CreateExerciseSession) -> Result<ExerciseSession> {
        let id = Uuid::new_v4();
        let started_on = Utc::now();
        sqlx::query(
            "INSERT INTO exercise_session
            (id, started_on, command_args, exercise_id, folder_path)
            VALUES
            (?, ?, ?, ?, ?)",
        )
        .bind(id)
        .bind(started_on)
        .bind(payload.command_args.clone())
        .bind(payload.exercise_id)
        .bind(payload.folder_path.clone())
        .execute(&self.pool)
        .await?;

        Ok(ExerciseSession {
            id,
            exercise_id: payload.exercise_id,
            command_args: payload.command_args,
            started_on,
            updated_on: started_on,
            folder_path: payload.folder_path,
            completed_on: None
        })
    }

    pub async fn update_exercise_session(&self, payload: UpdateExerciseSession) -> Result<()> {
        let mut sql = String::from("UPDATE exercise_session SET ");
        let mut sets = Vec::new();

        if payload.folder_path.is_some() {
            sets.push("folder_path = ?");
        }
        if payload.completed_on.is_some() {
            sets.push("completed_on = ?");
        }

        if sets.is_empty() {
            return Ok(());
        }

        sql.push_str(&sets.join(", "));
        sql.push_str(" WHERE id = ?");

        let mut query = sqlx::query(&sql);

        if let Some(fp) = payload.folder_path {
            query = query.bind(fp);
        }
        if let Some(co) = payload.completed_on {
            query = query.bind(co);
        }

        query = query.bind(payload.exercise_id);
        query.execute(&self.pool).await?;

        Ok(())
    }
}
