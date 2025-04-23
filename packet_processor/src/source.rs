use crate::{models::Packet, utils::random_alphabetic_string_capitalized};
use bincode::{config::Configuration, Decode};
use rand::{rng, Rng};

pub enum State {
    Init
}

pub struct Source {
    config: Configuration,
    state: i64,
}

impl Iterator for Source {
    type Item = Vec<u8>;

    fn next(&mut self) -> Option<Self::Item> {

        if self.state == 0 {
            let packet = Packet::NewPlayer { 
                id: rng().random(),
                name: random_alphabetic_string_capitalized(10)
            };
            let data = bincode::encode_to_vec(packet, self.config).unwrap();
            
            self.state += 1;

            return Some(data)
        }
        
        if self.state == 1 {
            let packet = Packet::NewPlayer { 
                id: rng().random(),
                name: random_alphabetic_string_capitalized(10)
            };
            let data = bincode::encode_to_vec(packet, self.config).unwrap();

            self.state += 1;

            return Some(data)
        }

        None
    }
}

impl Source {
    pub fn new() -> Self {
        let config = bincode::config::standard();

        Self {
            config,
            state: 0
        }
    }
}