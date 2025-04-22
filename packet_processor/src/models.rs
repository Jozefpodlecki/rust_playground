use bincode::{Decode, Encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub enum Packet {
    NewPlayer {
        id: u64,
        name: String,
    }
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Clone)]
pub struct Boss {
    pub id: u64,
    pub name: String,
}

pub struct Settings {
    port: u16
}