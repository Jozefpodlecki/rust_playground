use std::default;

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

pub trait PacketHandler {
    fn handle(&self, kind: PacketType, data: Vec<u8>) -> Result<()>;
}

#[derive(Debug, Default)]
pub struct DefaultPacketHandler<SH, NP>
where
    SH: StartHandler,
    NP: NewPlayerHandler
{
    start_handler: Option<SH>,
    new_player_handler: Option<NP>,
}

impl<SH, NP> PacketHandler for DefaultPacketHandler<SH, NP>
where
    SH: StartHandler,
    NP: NewPlayerHandler
{
    fn handle(&self, kind: PacketType, data: Vec<u8>) -> Result<()> {

        match kind {
            PacketType::Start => {
                if let Some(handler) = &self.start_handler {
                    handler.process();
                }
            },
            PacketType::NewPlayer => {
                if let Some(handler) = &self.new_player_handler {

                    handler.process();
                }
            },
            PacketType::NewNpc => {
                if let Some(handler) = &self.start_handler {
                    handler.process();
                }
            },
            PacketType::Attack => {
                if let Some(handler) = &self.start_handler {
                    handler.process();
                }
            },
            PacketType::End => {
                if let Some(handler) = &self.start_handler {
                    handler.process();
                }
            },
        }

        Ok(())
    }
}

impl<SH, NP> DefaultPacketHandler<SH, NP>
where
    SH: StartHandler,
    NP: NewPlayerHandler
{
    pub fn new() -> Self {
        Self {
            start_handler: None,
            new_player_handler: None
        }
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

impl NewPlayerPacket {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ..Default::default()
        })
    }
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