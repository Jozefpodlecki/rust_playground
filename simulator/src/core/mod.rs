pub mod player;
pub mod types;
pub mod event;
pub mod template;
pub mod utils;
pub mod bard;
use std::sync::{Arc, Barrier, RwLock};

use crossbeam::channel::unbounded;

use rand::{rng, Rng};

use crate::core::{bard::{BardSimulatorPlayer}, event::SimulatorEvent, player::*, types::*};

pub struct Simulator {
    parties: Vec<SimulatorParty>
}

impl Simulator {
    pub fn new(template: EncounterTemplate) -> Self {
        let mut parties = vec![];

        for party in template.parties {
            let mut simulator_party = SimulatorParty {
                id: party.id,
                members: vec![]
            };

            for member in party.members.clone() {

                let args = SimulatorPlayerArgs {
                    id: member.id,
                    name: member.name
                };

                let player: Box<dyn SimulatorPlayer> = match member.class_id {
                    player::Class::Bard => Box::new(BardSimulatorPlayer::new(args)),
                    player::Class::Berserk => Box::new(BerserkerSimulatorPlayer::new(args)),
                    player::Class::Sorceress => Box::new(SorceressSimulatorPlayer::new(args)),
                    player::Class::Gunslinger => Box::new(GunslingerSimulatorPlayer::new(args)),
                };
                simulator_party.members.push(player);
            }

            parties.push(simulator_party);
        }

        Self {
            parties
        }
    }

    pub fn setup(&mut self) {

    }

    pub fn run(mut self) {
        let (tx, rx) = unbounded::<SimulatorEvent>();

        let boss = SimulatorBoss {
            current_hp: 0,
            hp_bars: 0,
            id: 0,
            max_hp: 10
        };

        let context = SimulatorContext {
            barrier: Arc::new(Barrier::new(5)),
            current_boss: RwLock::new(boss)
        };
        let context = Arc::new(context);

        for party in self.parties.iter_mut() {
            for player in party.members.iter_mut() {
                player.run(context.clone(), tx.clone());
            }
        }

        while let Ok(event) = rx.recv() {
            match event {
                SimulatorEvent::NewPlayer {  } => {

                },
                SimulatorEvent::NewParty {  } => {

                },
                SimulatorEvent::NewBoss {  } => {

                },
                SimulatorEvent::RaidComplete {  } => 
                {

                },
                SimulatorEvent::BossDead {  } => {

                },
                SimulatorEvent::SkillDamage { source_id, skill_id, damage, target_id } => {
                    
                },
            }
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