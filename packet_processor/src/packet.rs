use bincode::{config::Configuration, Decode, Encode};
use packet_processor_macro::GenerateTraits;
use serde::{Deserialize, Serialize};
use anyhow::*;

#[derive(GenerateTraits, Debug, Clone)]
pub enum PacketType {
    #[no_data]
    Start = 0,
    #[with_struct(NewPlayerPacket)]
    NewPlayer = 1,
    // #[with_struct(NewNpcPacket)]
    NewNpc = 2,
    // #[with_struct(AttackPacket)]
    Attack = 3,
    #[no_data]
    End = 4,
}

pub struct Start;

impl StartHandler for Start {
    fn process(&self) {
        todo!()
    }
}

pub struct NewPlayer;

impl NewPlayerHandler for NewPlayer {
    fn process(&self,data:NewPlayerPacket) {
        todo!()
    }
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