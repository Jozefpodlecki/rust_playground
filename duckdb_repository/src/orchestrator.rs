use anyhow::{Ok, Result};

use crate::simulator::Simulator;

pub struct Orchestrator {
    simulator: Simulator
}

impl Orchestrator {
    pub fn new(simulator: Simulator) -> Self {
        Self { simulator }
    }

    pub fn run(&mut self) -> Result<()> {

        self.simulator.setup();


        Ok(())
    }
}

fn create_database_and_insert_record() -> Result<()> {
   
    

   
    // let name = "Alice".to_string();

    // if player_repository.exists(&name)? {
    //     let player = Player {
    //         id: Uuid::now_v7(),
    //         name,
    //         class_id: 101,
    //         character_id: 1234,
    //         last_gear_score: 1700.0,
    //         created_on: Utc::now(),
    //         updated_on: Utc::now(),
    //     };

    //     player_repository.insert(&player)?;
    // }

    // let zone = Zone {
    //     id: 1,
    //     name: "test".into()
    // };

    // let raid = Raid {
    //     id: Uuid::now_v7(),
    //     name: "".into(),
    //     sub_name: None,
    //     created_on: Utc::now(),
    //     gate: 2,
    //     zone_ids: vec![]
    // };

    // let npc = Npc {
    //     id: Uuid::now_v7(),
    //     created_on: Utc::now(),
    //     name: "Test".into(),
    //     npc_type_id: 1,
    //     raid_id: Uuid::now_v7(),
    // };

    // npc_repository.insert(npc).unwrap();


    Ok(())
}
