use std::time::Duration;

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

#[derive(Default, Serialize, Deserialize, Clone)]
pub struct Settings {
    pub port: u16,
    pub summary_emit_interval: Duration
}