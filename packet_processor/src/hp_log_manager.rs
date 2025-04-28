use std::collections::{hash_map::Entry, HashMap};

use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
pub struct HpLogSession {
    session_id: Uuid,
    // entries: Vec<HpLogEntry>,
    current: HpLogEntry
}

pub struct Repository {}

impl Repository {
    pub fn new() -> Self {
        Self {
        }
    }

    pub async fn insert(&self, entity: HpLogEntry) {

    }
}

#[derive(Debug, Clone, Serialize)]
pub struct HpLogEntry {
    pub timestamp: i32,
    pub hp: i64,
    pub hp_percentage: f32,
}

pub struct HpLogManager  {
    sessions: HashMap<Uuid, HpLogSession>,
    log_repository: Repository
}

impl HpLogManager {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            log_repository: Repository::new()
        }
    }

    pub async fn insert(&mut self, session_id: Uuid, timestamp: i64, current_hp: u64, hp_percentage: f32) {
        let relative_timestamp_s = (timestamp / 1000) as i32;

        match self.sessions.entry(session_id) {
            Entry::Occupied(mut entry) => {
              
                let session = entry.get_mut();

                if session.current.timestamp == relative_timestamp_s {
                    session.current.hp = current_hp as i64;
                    session.current.hp_percentage = hp_percentage;
                }
                else {
                    // session.entries.push(session.current.clone());
                    self.log_repository.insert(session.current.clone()).await;
                    session.current.hp = current_hp as i64;
                    session.current.hp_percentage = hp_percentage;
                    session.current.timestamp = relative_timestamp_s;
                }
            },
            Entry::Vacant(entry) => {
                let session = HpLogSession {
                    session_id,
                    current: HpLogEntry {
                        timestamp: relative_timestamp_s,
                        hp: current_hp as i64,
                        hp_percentage,
                    },
                };

                entry.insert(session);
            },
        }

    }

    pub fn clear(&mut self) {
        self.sessions.clear();
    }

    pub fn flush(&mut self) {
        for session in self.sessions.values_mut() {
            // session.entries.push(session.current.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_insert_new_session() {
        let mut hp_log = HpLogManager::new();
        let session_id = Uuid::now_v7();
        hp_log.insert(session_id, 1000, 500, 80.0);
        let session = hp_log.sessions.get(&session_id).expect("Session not found");
        assert_eq!(session.session_id, session_id);
        assert_eq!(session.current.hp, 500);
        assert_eq!(session.current.hp_percentage, 80.0);
        assert_eq!(session.current.timestamp, 1);
    }

    #[test]
    fn test_flush() {
        let mut hp_log = HpLogManager::new();
        let session_id = Uuid::now_v7();
        hp_log.insert(session_id, 1000, 500, 80.0);
        hp_log.flush();
        let session = hp_log.sessions.get(&session_id).expect("Session not found");
    }

    #[test]
    fn test_insert_existing_session_same_timestamp() {
        let mut hp_log = HpLogManager::new();
        let session_id = Uuid::now_v7();
        hp_log.insert(session_id, 1000, 500, 80.0);
        hp_log.insert(session_id, 1000, 600, 90.0);
        let session = hp_log.sessions.get(&session_id).expect("Session not found");
        assert_eq!(session.current.hp, 600);
        assert_eq!(session.current.hp_percentage, 90.0);
        assert_eq!(session.current.timestamp, 1);
    }

    #[test]
    fn test_insert_existing_session_different_timestamp() {
        let mut hp_log = HpLogManager::new();
        let session_id = Uuid::now_v7();
        hp_log.insert(session_id, 1000, 500, 80.0);
        hp_log.insert(session_id, 2000, 600, 90.0);
        let session = hp_log.sessions.get(&session_id).expect("Session not found");
        assert_eq!(session.current.hp, 600);
        assert_eq!(session.current.hp_percentage, 90.0);
        assert_eq!(session.current.timestamp, 2);
    }

    #[test]
    fn test_clear_sessions() {
        let mut hp_log = HpLogManager::new();
        let session_id = Uuid::now_v7();
        hp_log.insert(session_id, 1000, 500, 80.0);
        hp_log.insert(session_id, 2000, 600, 90.0);
        assert!(hp_log.sessions.contains_key(&session_id));
        hp_log.clear();
        assert!(!hp_log.sessions.contains_key(&session_id));
    }
}