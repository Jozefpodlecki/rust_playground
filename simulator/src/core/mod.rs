pub mod player;
pub mod types;
pub mod event;
pub mod template;
pub mod utils;
pub mod bard;
pub mod boss;

use std::{collections::HashMap, sync::{Arc, Barrier, RwLock}};

use crossbeam::channel::unbounded;

use log::info;
use rand::{rng, Rng};

use crate::core::{bard::BardSimulatorPlayer, boss::SimulatorBoss, event::SimulatorEvent, player::*, types::*};

pub struct Simulator {
    boss: Option<SimulatorBoss>,
    player_ids: Vec<u64>,
    party_map: HashMap<u32, Vec<u64>>,
    parties: Vec<SimulatorParty>
}

impl Simulator {
    pub fn new(template: EncounterTemplate) -> Self {
        let mut parties = vec![];
        let mut player_ids = vec![];
        let mut party_map = HashMap::new();

        for party in template.parties {
            let mut simulator_party = SimulatorParty {
                id: party.id,
                members: vec![]
            };
            let member_ids: &mut Vec<u64> = party_map.entry(party.id).or_default();

            for member in party.members.clone() {

                let args = SimulatorPlayerArgs {
                    id: member.id,
                    name: member.name,
                    attack_power: member.attack_power,
                    cooldown_reduction: member.cooldown_reduction,
                    crit_damage: member.crit_damage,
                    crit_rate: member.crit_rate,
                    party_id: party.id
                };

                player_ids.push(member.id);
                member_ids.push(member.id);

                let player: Box<dyn SimulatorPlayer> = match member.class_id {
                    player::Class::Bard => Box::new(BardSimulatorPlayer::new(args)),
                    player::Class::Berserk => Box::new(BerserkerSimulatorPlayer::new(args)),
                    player::Class::Sorceress => Box::new(SorceressSimulatorPlayer::new(args)),
                    player::Class::Gunslinger => Box::new(GunslingerSimulatorPlayer::new(args)),
                    _ => panic!("Unknown class")
                };
                simulator_party.members.push(player);
            }

            parties.push(simulator_party);
        }

        let boss = SimulatorBoss::new(template.boss);

        Self {
            boss: Some(boss),
            player_ids,
            party_map,
            parties
        }
    }

    pub fn setup(&mut self) {

    }

    pub fn run(mut self) {
        let (tx, rx) = unbounded::<SimulatorEvent>();

        info!("Preparing boss...");
        let boss = self.boss.unwrap();

        let event = SimulatorEvent::NewBoss { id: boss.id };

        tx.send(event).unwrap();

        let context = SimulatorContext {
            party_map: self.party_map,
            player_ids: self.player_ids,
            barrier: Arc::new(Barrier::new(6)),
            current_boss: RwLock::new(boss)
        };

        let context = Arc::new(context);

        info!("Preparing parties...");

        for party in self.parties.iter_mut() {

            let mut members = vec![];

            for player in party.members.iter_mut() {

                let base = player.base();

                let event = SimulatorEvent::NewPlayer { 
                    id: base.id,
                    name: base.name.clone(),
                    class_id: base.class_id
                };

                members.push(base);

                tx.send(event).unwrap();

                player.run(context.clone(), rx.clone(), tx.clone());
            }

            let event = SimulatorEvent::NewParty { 
                id: party.id,
                members,
            };

            tx.send(event).unwrap();
        }

        let cloned = context.clone();
        cloned.current_boss.write().unwrap().run(context, rx.clone(), tx.clone());

        info!("Starting...");
        cloned.barrier.wait();

        while let Ok(event) = rx.recv() {
           info!("Received: {event:?}");
        }
    }
}


#[cfg(test)]
mod tests {
    use crate::core::template::template_3dps_1support;

    use super::*;

    #[test]
    pub fn should_work() {
        let template = template_3dps_1support();

        let mut simulator = Simulator::new(template);
        simulator.setup();
        simulator.run();
    }
}