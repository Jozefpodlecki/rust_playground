use std::collections::HashMap;

use chrono::{DateTime, Utc};

#[derive(Hash)]
pub struct Entity {
    pub id: u8,
    pub name: &'static str,
}

impl PartialEq for Entity {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Eq for Entity {}

fn main() {
   
    let entity = Entity {
        id: 1,
        name: "player_1"
    };

   let mut map: HashMap<&Entity, DateTime<Utc>> = HashMap::new();

    map.insert(&entity, Utc::now());
}
