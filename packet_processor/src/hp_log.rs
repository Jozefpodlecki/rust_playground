use uuid::Uuid;

pub struct HpLogEntry {
    
}

pub struct HpLog {
    session_id: Uuid,
    previous: Option<HpLogEntry>
}

impl HpLog {
    pub fn new(session_id: Uuid) -> Self {
        Self {
            session_id,
            previous: None
        }
    }

    pub fn insert(&mut self, timestamp: i64, current_hp: u64, hp_percentage: f32) {
        if let Some(entry) = self.previous.take() {
            
        }
    }

    pub fn flush(&mut self) {
        if let Some(entry) = self.previous.take() {

        }
    }
}