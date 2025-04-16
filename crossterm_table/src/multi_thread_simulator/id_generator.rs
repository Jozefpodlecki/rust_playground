use std::collections::{HashMap, HashSet};

use rand::{distr::uniform::SampleRange, rng, rngs::ThreadRng, Rng};

const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyz";

#[derive(Default)]
pub struct IdGenerator {
    rng: ThreadRng,
    buff_instance_ids: HashSet<u32>,
    npc_ids: HashSet<u64>,
    player_ids: HashSet<u64>,
    player_names: HashSet<String>,
    party_ids: HashSet<u64>,
}

impl IdGenerator {
    pub fn new() -> Self {
        let rng = rng();
        Self {
            rng,
            ..Default::default()
        }
    }

    pub fn next_buff_instance_id(&mut self) -> u32 {
        let mut rng = rng();
        let mut id = rng.random_range(1000..9999);

        while(!self.buff_instance_ids.insert(id)) {
            id = rng.random_range(1000..9999);
        }

        id
    }

    pub fn next_npc_id(&mut self) -> u64 {
        let mut rng = rng();
        let mut id = rng.random_range(1000..9999);

        while(!self.npc_ids.insert(id)) {
            id = rng.random_range(1000..9999);
        }

        id
    }

    pub fn next_party_id(&mut self) -> u64 {
        let mut rng = rng();
        let mut id = rng.random_range(1000..9999);

        while(!self.party_ids.insert(id)) {
            id = rng.random_range(1000..9999);
        }

        id
    }

    pub fn next_player_name(&mut self, length: usize) -> String {
        let mut name: String = Self::random_name(length);

        while !self.player_names.insert(name.clone()) {
            name = Self::random_name(length);
        }
    
        name
    }

    fn random_name(length: usize) -> String {
        let mut rng = rng();

        let mut name: String = (0..length)
        .map(|_| CHARSET[rng.random_range(0..CHARSET.len())] as char)
        .collect();

        let first_char = name.get_mut(0..1).unwrap();
        first_char.make_ascii_uppercase();

        name
    }
    
    pub fn next_player_id(&mut self) -> u64 {
        let mut rng = rng();
        let mut id = rng.random_range(1000..9999);

        while(!self.player_ids.insert(id)) {
            id = rng.random_range(1000..9999);
        }

        id
    }

    pub fn next_bool(&mut self, ratio: f64) -> bool {
        self.rng.random_bool(ratio)
    }

    pub fn next_f32(&mut self, range: impl SampleRange<f32>) -> f32 {
        self.rng.random_range(range)
    }
}