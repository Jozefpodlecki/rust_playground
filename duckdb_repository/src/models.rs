use chrono::{DateTime, Utc};
use uuid::Uuid;


pub struct Player {
    pub id: Uuid,
    pub name: String,
    pub created_on: DateTime<Utc>,
}