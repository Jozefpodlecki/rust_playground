use sqlx::{Pool, Sqlite};

use crate::models::Exercise;

pub struct ExerciseManager {
    pool: Pool<Sqlite>
}

impl ExerciseManager {
    pub fn new(pool: Pool<Sqlite>) -> Self {
        Self { pool }
    }

    pub fn get_exercises(&self) -> Vec<Exercise> {
        vec![]
    }


}