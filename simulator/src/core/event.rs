pub enum SimulatorEvent {
    NewPlayer {

    },
    NewParty {

    },
    NewBoss {

    },
    RaidComplete {

    },
    BossDead {

    },
    SkillDamage {
        source_id: u64, 
        skill_id: u32,
        damage: i64,
        target_id: u64
    }
}
