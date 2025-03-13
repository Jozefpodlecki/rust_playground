use std::{sync::mpsc, thread, time::Duration};

use bincode::{config::Configuration, Decode};

use crate::{game_state::{self, GameState}, packet::{AttackPacket, NewNpcPacket, NewPlayerPacket, PacketType}};


pub struct Processor {

}

impl Processor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn run(&self) {
        let (tx, rx) = mpsc::channel::<(PacketType, Vec<u8>)>();
        let config = bincode::config::standard();
        let mut game_state = GameState::new();

        thread::spawn(move || {
            
            
            let kind = PacketType::NewPlayer;
            let packet = NewPlayerPacket {
                id: 1,
                name: "Player".into(),
            };
            let data = bincode::encode_to_vec(packet, config).unwrap();
            tx.send((kind, data)).unwrap();

            let kind = PacketType::NewNpc;
            let packet = NewNpcPacket {
                id: 2,
                name: "Boss".into(),
            };
            let data = bincode::encode_to_vec(packet, config).unwrap();
            tx.send((kind, data)).unwrap();

            let kind = PacketType::Start;
            tx.send((kind, vec![])).unwrap();

            let duration: Duration = Duration::from_secs(1);
            let mut total_damage = 0;

            loop {
                if total_damage > 10 {
                    break;
                }

                let kind = PacketType::Attack;
                let packet = AttackPacket {
                    source_id: 1,
                    target_id: 2,
                    damage: 1
                };
                let data = bincode::encode_to_vec(packet, config).unwrap();
                tx.send((kind, data)).unwrap();
                thread::sleep(duration);
            }
            
            let kind = PacketType::End;
            tx.send((kind, vec![])).unwrap();

        });

        while let Ok((kind, data)) = rx.recv() {
            println!("{:?}", kind);

            match kind {
                PacketType::NewPlayer => {
                    if let Some(packet) = parse::<NewPlayerPacket>(&data, config) {
                        game_state.on_new_player(packet);
                    }
                },
                PacketType::NewNpc => {
                    if let Some(packet) = parse::<NewNpcPacket>(&data, config) {
                        game_state.on_new_npc(packet);
                    }
                },
                PacketType::Attack => {
                    if let Some(packet) = parse::<AttackPacket>(&data, config) {
                        game_state.on_attack(packet);
                    }
                },
                PacketType::Start => {

                },
                PacketType::End => {

                },
            }
        }
    }
}

fn parse<T>(data: &[u8], config: Configuration) -> Option<T>
    where T: bincode::de::Decode<()> {
    let (packet, _): (T, _)  = bincode::decode_from_slice(&data, config).unwrap();
    Some(packet)
}