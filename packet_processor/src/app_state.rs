use std::{cell::RefCell, collections::HashMap, rc::Rc};

use chrono::{DateTime, Duration, Utc};
use once_cell::sync::Lazy;
use uuid::{Timestamp, Uuid};

use crate::{hp_log_manager::HpLogManager, models::*};

pub struct Skill {
    pub id: u32,
    pub class_id: u32
}

pub static SKILL_IDS: Lazy<HashMap<u32, Skill>> = Lazy::new(|| {
    vec![
        (1, Skill {
            id: 1,
            class_id: 101   
        })
    ].into_iter().collect()
});

pub struct AppState {
    id: Uuid,
    started_on: DateTime<Utc>,
    duration: Duration,
    entities_by_id: HashMap<u64, Rc<RefCell<Entity>>>,
    entities_by_character_id: HashMap<u64, Rc<RefCell<Entity>>>,
    raid_stats: RaidStats,
    stats: HashMap<u64, EntityStats>,
    hp_log: HpLogManager,
    has_ended: bool
}

impl AppState {
    pub fn new() -> Self {
        Self {
            id: Uuid::nil(),
            started_on: DateTime::<Utc>::MIN_UTC,
            duration: Duration::zero(),
            entities_by_id: HashMap::new(),
            entities_by_character_id: HashMap::new(),
            raid_stats: RaidStats::default(),
            stats: HashMap::new(),
            hp_log: HpLogManager::new(),
            has_ended: false,
        }
    }

    pub fn new_player(&mut self, id: u64, character_id: u64, name: String, gear_score: f32) {
        let entity = Entity {
            id,
            kind: EntityType::Player,
            character_id: Some(character_id),
            owner_id: None,
            name,
            gear_score: gear_score
        };

        let entity = Rc::new(RefCell::new(entity));
        self.entities_by_id.insert(id, entity.clone());
        self.entities_by_character_id.insert(character_id, entity);
        self.stats.insert(id, EntityStats::default());
    }

    fn new_player_from_party(character_id: u64, name: String, gear_score: f32) -> Rc<RefCell<Entity>> {
        let entity = Entity {
            id: 0,
            kind: EntityType::Player,
            character_id: Some(character_id),
            owner_id: None,
            name,
            gear_score: gear_score
        };

        let entity = Rc::new(RefCell::new(entity));
        entity
    }

    fn new_unknown(id: u64, skill_id: Option<u32>) -> Rc<RefCell<Entity>> {
        let entity = Entity {
            id: id,
            kind: EntityType::Unknown,
            character_id: None,
            owner_id: None,
            name: "".into(),
            gear_score: 0.0
        };

        if let Some(skill) = skill_id.and_then(|id| SKILL_IDS.get(&id)) {
            
        }

        let entity = Rc::new(RefCell::new(entity));
        entity
    }

    pub fn new_boss(&mut self, id: u64, name: String) {
        let entity = Entity {
            id,
            kind: EntityType::Boss,
            name,
            ..Default::default()
        };

        let entity = Rc::new(RefCell::new(entity));
        self.entities_by_id.insert(id, entity);
        self.stats.insert(id, EntityStats::default());
    }

    pub fn add_buff(&mut self, target_id: u64, effect: StatusEffect) {

        let entity = self.resolve_entity(target_id, effect.target);


    }

    fn resolve_entity(&mut self, target_id: u64, buff_target: BuffTarget) -> Rc<RefCell<Entity>> {
        let lookup = if buff_target == BuffTarget::Party {
            self.entities_by_character_id.get(&target_id)
        } else {
            self.entities_by_id.get(&target_id)
        };
    
        match lookup {
            Some(entity) => {
                let owner_id = entity.borrow().owner_id;
                if let Some(owner_id) = owner_id {
                    self.entities_by_id
                        .get(&owner_id)
                        .cloned()
                        .unwrap_or_else(|| {
                            let unknown = Self::new_unknown(owner_id, None);
                            self.entities_by_id.insert(owner_id, unknown.clone());
                            unknown
                        })
                } else {
                    entity.clone()
                }
            }
            None => {
                let unknown = Self::new_unknown(target_id, None);
                if effect.target == BuffTarget::Party {
                    self.entities_by_character_id.insert(target_id, unknown.clone());
                }
                self.entities_by_id.insert(target_id, unknown.clone());
                
                unknown
            }
        }
    }

    pub fn new_party(&mut self, id: u32, members: &[PartyMember]) {
        
        for member in members {
            let entity = self.entities_by_character_id
                .entry(member.character_id)
                .and_modify(|pr| {
                    let mut entity = pr.borrow_mut();
                    entity.gear_score = member.gear_score;
                })
                .or_insert_with(|| Self::new_player_from_party(member.character_id, member.name.clone(), member.gear_score));

            
        }
    }

    pub async fn on_damage(&mut self,
        skill_id: u32,
        source_id: u64,
        target_id: u64,
        value: u64,
        current_hp: u64,
        hp: u64) {

        let now = Utc::now();

        if self.started_on == DateTime::<Utc>::MIN_UTC {
            self.started_on = now;
            {
                let duration = self.started_on.signed_duration_since(chrono::DateTime::<Utc>::from(std::time::UNIX_EPOCH));
                let ts = Timestamp::from_unix(&uuid::NoContext, duration.num_seconds() as u64, duration.subsec_nanos() as u32);
                self.id = Uuid::new_v7(ts);

                self.hp_log.clear();
            }
        }
        else {
            self.duration = now - self.started_on;
        }

        let source = self.entities_by_id.entry(source_id).or_insert_with(|| Self::new_unknown(source_id, Some(skill_id)));
        let target = self.entities_by_id.entry(target_id).or_insert_with(|| Self::new_unknown(target_id, None));
        
        {
            let target_stats = self.stats.entry(target_id).or_default();
            target_stats.current_hp = current_hp;
            target_stats.hp = hp;
        }

        let source_stats = self.stats.entry(source_id).or_default();

        let duration_seconds = self.duration.num_seconds();

        source_stats.total_damage += value;
        self.raid_stats.total_damage += value;

        if duration_seconds > 0 {
            self.raid_stats.dps = self.raid_stats.total_damage as f32 / duration_seconds as f32;

            source_stats.dps = source_stats.total_damage as f32 / duration_seconds as f32;
            source_stats.total_damage_percentage = source_stats.total_damage as f32 / self.raid_stats.total_damage as f32;
        }

        let hp_percentage = current_hp as f32 / hp as f32;
        self.hp_log.insert(self.id,duration_seconds, current_hp, hp_percentage).await;
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
            id: &self.id,
            started_on: &self.started_on,
            players: vec![]
        })
    }

    pub fn reset(&mut self) {
        self.id = Uuid::nil();
        self.started_on = DateTime::<Utc>::MIN_UTC;
        self.duration = Duration::zero();
        self.has_ended = false;
        self.entities_by_character_id.clear();
        self.entities_by_id.clear();
    }
}