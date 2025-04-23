use std::collections::HashMap;

use crate::models::{Entity, Settings};

pub struct AppState {
    entities: HashMap<u64, Entity>
}

impl AppState {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new()
        }
    }

    pub fn new_player(&mut self, id: u64, character_id: u64, name: String) {

    }

    pub fn get_summary(&self, settings: &Settings) {

    }
}