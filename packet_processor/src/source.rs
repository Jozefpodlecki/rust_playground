use crate::{models::{Boss, Packet, PartyMember, Player}, utils::random_alphabetic_string_capitalized};
use bincode::{config::Configuration, Decode};
use rand::{rng, Rng};
use tokio::sync::mpsc::UnboundedSender;

pub enum State {
    Init
}

pub struct Party {

}

pub struct AttackResult {
    pub damage: u64
}

pub struct Source {
    config: Configuration,
    state: i64,
    boss: Boss,
    players: Vec<Player>,
    parties: Vec<Party>
}

impl Source {
    pub fn new() -> Self {
        let config = bincode::config::standard();

        Self {
            config,
            state: 0,
            boss: Boss::default(),
            players: vec![],
            parties: vec![]
        }
    }

    pub fn run(&mut self, tx: UnboundedSender<Vec<u8>>) {
        let mut players = Vec::new();
        
        let dps = self.spawn_player();
        players.push(dps.0);
        tx.send(dps.1).unwrap();

        let dps = self.spawn_player();
        players.push(dps.0);
        tx.send(dps.1).unwrap();

        let dps = self.spawn_player();
        players.push(dps.0);
        tx.send(dps.1).unwrap();

        let support = self.spawn_player();
        players.push(support.0);
        tx.send(support.1).unwrap();

        let party = self.spawn_party(&players);
        tx.send(party.1).unwrap();
        self.players.extend(players);
        
        let mut players = Vec::new();

        let dps = self.spawn_player();
        players.push(dps.0);
        tx.send(dps.1).unwrap();

        let dps = self.spawn_player();
        players.push(dps.0);
        tx.send(dps.1).unwrap();

        let dps = self.spawn_player();
        players.push(dps.0);
        tx.send(dps.1).unwrap();

        let support = self.spawn_player();
        players.push(support.0);
        tx.send(support.1).unwrap();

        let party = self.spawn_party(&players);
        tx.send(party.1).unwrap();
        self.players.extend(players);

        let boss = self.spawn_boss();
        tx.send(boss.1).unwrap();
        self.boss = boss.0;
        
        let rng = rng();
        let players_length = self.players.len();

        loop {

            for player in self.players.iter() {
                let result = self.perform_attack(&player, &self.boss);
                tx.send(result.1).unwrap();

                self.boss.hp = result.0.damage;
            }
        }
        
    }

    pub fn spawn_party(&mut self, players: &[Player]) -> (Party, Vec<u8>) {
        
        let packet = Packet::Party {
            id: rng().random(),
            members: players.iter().map(|pr| PartyMember {
                character_id: pr.character_id,
                name: pr.name.to_string()
            }).collect()
        };
        let data = bincode::encode_to_vec(packet, self.config).unwrap();

        let party = Party {

        };

       (party, data)
    }

    pub fn perform_attack(&self, player: &Player, boss: &Boss) -> (AttackResult, Vec<u8>) {

        let result = AttackResult {
            damage: rng().random_range(10..99),
        };

        let packet = Packet::Damage { 
            skill_id: 1,
            source_id: player.id,
            target_id: boss.id,
            value: 100
        };
        let data = bincode::encode_to_vec(packet, self.config).unwrap();

        (result, data)
    }

    pub fn spawn_boss(&self) -> (Boss, Vec<u8>) {
        let boss = Boss {
            id: rng().random(),
            name: random_alphabetic_string_capitalized(10),
            hp: 1000,
            current_hp: 1000
        };

        let packet = Packet::NewBoss {
            id: boss.id,
            name: boss.name.clone()
        };
        let data = bincode::encode_to_vec(packet, self.config).unwrap();

        (boss, data)
    }

    pub fn spawn_player(&self) -> (Player, Vec<u8>) {

        let player = Player {
            id: rng().random(),
            character_id: rng().random(),
            name: random_alphabetic_string_capitalized(10)
        };
      
        let packet = Packet::NewPlayer { 
            id: player.id,
            character_id: player.character_id,
            name: player.name.clone()
        };
        let data = bincode::encode_to_vec(packet, self.config).unwrap();

        (player, data)
    }
}