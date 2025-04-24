use uuid::Uuid;

pub struct HpLog {
    session_id: Uuid
}

impl HpLog {
    pub fn new(session_id: Uuid) -> Self {
        Self {
            session_id
        }
    }

    pub fn insert(&mut self, timestamp: i64, current_hp: u64, hp_percentage: f32) {

    }
}