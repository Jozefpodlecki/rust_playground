use std::{collections::HashMap, default};

use chrono::{DateTime, Utc};
use uuid::{Timestamp, Uuid};

use crate::models::{EncounterFragment, Entity, EntityStats, EntityType, PartyMember, Settings};

pub struct AppState {
    id: Uuid,
    started_on: DateTime<Utc>,
    entities: HashMap<u64, Entity>,
    stats: HashMap<u64, EntityStats>,
    has_ended: bool
}

impl AppState {
    pub fn new() -> Self {
        Self {
            id: Uuid::nil(),
            started_on: DateTime::<Utc>::MIN_UTC,
            entities: HashMap::new(),
            stats: HashMap::new(),
            has_ended: false,
        }
    }

    pub fn new_player(&mut self, id: u64, character_id: u64, name: String) {
        let entity = Entity {
            id,
            kind: EntityType::Player,
            character_id: Some(character_id),
            name,
        };

        self.entities.insert(id, entity);
    }

    pub fn new_boss(&mut self, id: u64, name: String) {
        let entity = Entity {
            id,
            kind: EntityType::Boss,
            name,
            ..Default::default()
        };

        self.entities.insert(id, entity);
    }

    pub fn new_party(&mut self, id: u32, members: &[PartyMember]) {
    }

    pub fn on_damage(&mut self, skill_id: u32, source_id: u64, target_id: u64, value: u64) {

        if self.started_on == DateTime::<Utc>::MIN_UTC {
            self.started_on = Utc::now();
            let duration = self.started_on.signed_duration_since(chrono::DateTime::<Utc>::from(std::time::UNIX_EPOCH));
            let ts = Timestamp::from_unix(&uuid::NoContext, duration.num_seconds() as u64, duration.subsec_nanos() as u32);
            self.id = Uuid::new_v7(ts);
        }

    }

    pub fn on_raid_end(&mut self) {
        self.has_ended = true;
    }

    pub fn get_summary(&self, settings: &Settings) -> Option<EncounterFragment> {
        
        if self.has_ended {
            return None;
        }

        if self.id == Uuid::nil() {
            return None;
        }

        Some(EncounterFragment { 
            id: self.id,
            started_on: self.started_on,
            players: vec![]
        })
    }

    pub fn reset(&mut self) {
        self.id = Uuid::nil();
        self.started_on = DateTime::<Utc>::MIN_UTC;
        self.has_ended = false;
    }
}