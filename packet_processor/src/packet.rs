use bincode::{config::Configuration, Decode, Encode};
use packet_processor_macro::connect_packet_to_structs;
use serde::{Deserialize, Serialize};
use anyhow::*;

#[derive(Debug, Clone)]
pub enum PacketType {
    Start = 0,
    NewPlayer = 1,
    NewNpc = 2,
    Attack = 3,
    End = 4,
}

pub trait PacketHandler {
    fn handle(&self, data: Vec<u8>) -> Result<()>;
}

impl PacketHandler for PacketType {
    fn handle(&self, data: Vec<u8>) -> Result<()> {

        Ok(())
    }
} 

trait Trait<T> {
    fn do_something(&self, value: T);
}


#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct NewPlayerPacket {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct NewNpcPacket {
    pub id: u64,
    pub name: String,
}

#[derive(Debug, Encode, Decode, Serialize, Deserialize, Default, Clone)]
pub struct AttackPacket {
    pub source_id: u64,
    pub damage: i64,
    pub target_id: u64
}

// connect_packet_to_structs!(PacketType);