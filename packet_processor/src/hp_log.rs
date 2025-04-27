use std::collections::HashMap;

use uuid::Uuid;

pub struct HpLogSession {
    session_id: Uuid,
    entries: Vec<HpLogEntry>,
    current: HpLogEntry
}

pub struct HpLogEntry {
    pub timestamp: i32,
    pub hp: i64,
    pub hp_percentage: f32,
}

pub struct HpLog {
    sessions: HashMap<Uuid, HpLogSession>
}

impl HpLog {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new()
        }
    }

    pub fn insert(&mut self, session_id: Uuid, timestamp: i64, current_hp: u64, hp_percentage: f32) {
        let relative_timestamp_s = (timestamp / 1000) as i32;

        let session = self.sessions.entry(session_id).or_insert_with(|| HpLogSession {
            session_id,
            entries: Vec::new(),
            current: HpLogEntry {
                timestamp: relative_timestamp_s,
                hp: current_hp as i64,
                hp_percentage,
            },
        });

        if session.entries.is_empty() || session.entries.last().unwrap().timestamp != relative_timestamp_s {
            session.entries.push(HpLogEntry {
                timestamp: relative_timestamp_s,
                hp: current_hp as i64,
                hp_percentage,
            });
        } else {
            let last_entry = session.entries.last_mut().unwrap();
            last_entry.hp = current_hp as i64;
            last_entry.hp_percentage = hp_percentage;
        }

        session.current.hp = current_hp as i64;
        session.current.hp_percentage = hp_percentage;
        session.current.timestamp = relative_timestamp_s;
    }

    pub fn clear(&mut self) {
        self.sessions.clear();
    }

    pub fn flush(&mut self) {
        
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_insert_new_session() {
        let mut hp_log = HpLog::new();
        let session_id = Uuid::now_v7();
        hp_log.insert(session_id, 1625254500000, 100, 0.75);
        let session = hp_log.sessions.get(&session_id).expect("Session should exist");
        assert_eq!(session.entries.len(), 1);
        assert_eq!(session.entries[0].timestamp, 1625254500);
        assert_eq!(session.entries[0].hp, 100);
        assert_eq!(session.entries[0].hp_percentage, 0.75);
    }

    #[test]
    fn test_insert_update_existing_entry() {
        let mut hp_log = HpLog::new();
        let session_id = Uuid::now_v7();
        hp_log.insert(session_id, 1625254500000, 100, 0.75);
        hp_log.insert(session_id, 1625254500000, 120, 0.80);
        let session = hp_log.sessions.get(&session_id).expect("Session should exist");
        assert_eq!(session.entries.len(), 1);
        assert_eq!(session.entries[0].hp, 120);
        assert_eq!(session.entries[0].hp_percentage, 0.80);
    }

    #[test]
    fn test_insert_with_different_timestamps() {
        let mut hp_log = HpLog::new();
        let session_id = Uuid::now_v7();
        hp_log.insert(session_id, 1625254500000, 100, 0.75);
        hp_log.insert(session_id, 1625254600000, 150, 0.85);
        let session = hp_log.sessions.get(&session_id).expect("Session should exist");
        assert_eq!(session.entries.len(), 2);
        assert_eq!(session.entries[1].timestamp, 1625254600);
        assert_eq!(session.entries[1].hp, 150);
        assert_eq!(session.entries[1].hp_percentage, 0.85);
    }

    #[test]
    fn test_multiple_sessions() {
        let mut hp_log = HpLog::new();
        let session_id1 = Uuid::now_v7();
        let session_id2 = Uuid::now_v7();
        hp_log.insert(session_id1, 1625254500000, 100, 0.75);
        hp_log.insert(session_id1, 1625254600000, 150, 0.85);
        hp_log.insert(session_id2, 1625254700000, 200, 0.95);
        let session1 = hp_log.sessions.get(&session_id1).expect("Session 1 should exist");
        assert_eq!(session1.entries.len(), 2);
        let session2 = hp_log.sessions.get(&session_id2).expect("Session 2 should exist");
        assert_eq!(session2.entries.len(), 1);
        assert_eq!(session2.entries[0].hp, 200);
    }
}
