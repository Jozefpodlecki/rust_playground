use std::{io::{stdout, Write}, time::{Duration, Instant}};

use bevy::{hierarchy::BuildChildren, time::Time, transform::commands::BuildChildrenTransformExt, utils::HashMap};
use bevy_ecs::prelude::*;
use crossterm::{cursor::MoveTo, style::Print, terminal::{Clear, ClearType}, ExecutableCommand};
use rand::{rng, Rng};

use crate::{components::*, utils::{format_number, random_alphabetic_string_capitalized}};

pub fn setup(mut commands: Commands) {
    commands.insert_resource(RaidClear(false));
    commands.insert_resource(PhaseIndex(0));

    let mut stdout = stdout();
    let _ = stdout.execute(Clear(ClearType::All));
    commands.insert_resource(StdoutResource(stdout));

    let boss = commands.spawn((
        Boss,
        Name("Behemoth".to_string()),
        Health::new(500_000_000_000, 500),
    )).id();
    let mut rng = rng();

    for party_id in 0..4 {
        let player_id = rng.random_range(1000..9000);
        let party = commands.spawn(Party(party_id)).id();

        let name = random_alphabetic_string_capitalized(10);
        let mut player = commands.spawn((
            Name(name),
            Player(player_id),
            Health::new(300_000, 1),
            AttackPower(100000),
            CritRate(0.1),
            Swiftness(600),
            Buffs(vec![Buff::permanent(BuffType::IncreaseCrit, 0.1), Buff::permanent(BuffType::IncreaseHp, 0.1)]),
            Cooldowns(HashMap::new()),
            DamageMeter(0),
            Class::Sorceress,
            SkillSet::sorceress(),
            AttackTarget(boss),
            DamageDealer,
            CastingState::default(),
        ));
        player.set_parent(party);

        let name = random_alphabetic_string_capitalized(10);
        let mut player = commands.spawn((
            Name(name),
            Player(player_id),
            Health::new(300_000, 1),
            AttackPower(100000),
            CritRate(0.1),
            Swiftness(50),
            Buffs(vec![Buff::permanent(BuffType::IncreaseCrit, 0.1), Buff::permanent(BuffType::IncreaseHp, 0.1)]),
            Cooldowns(HashMap::new()),
            DamageMeter(0),
            Class::Aeromancer,
            SkillSet::aeromancer(),
            AttackTarget(boss),
            DamageDealer,
            CastingState::default(),
        ));
        player.set_parent(party);

        let name = random_alphabetic_string_capitalized(10);
        let mut player = commands.spawn((
            Name(name),
            Player(player_id),
            Health::new(300_000, 1),
            AttackPower(100000),
            CritRate(0.1),
            Swiftness(50),
            Buffs(vec![Buff::permanent(BuffType::IncreaseCrit, 0.1), Buff::permanent(BuffType::IncreaseHp, 0.1)]),
            Cooldowns(HashMap::new()),
            DamageMeter(0),
            Class::Berserker,
            SkillSet::berserker(),
            AttackTarget(boss),
            DamageDealer,
            CastingState::default(),
        ));
        player.set_parent(party);

        let name = random_alphabetic_string_capitalized(10);
        let mut player = commands.spawn((
            Name(name),
            Player(player_id),
            Health::new(300_000, 1),
            AttackPower(100000),
            CritRate(0.1),
            Swiftness(1700),
            Buffs(vec![Buff::permanent(BuffType::IncreaseSwift, 0.1), Buff::permanent(BuffType::IncreaseHp, 0.1)]),
            Cooldowns(HashMap::new()),
            DamageMeter(0),
            Class::Bard,
            SkillSet::bard(),
            AttackTarget(boss),
            Support,
            CastingState::default(),
        ));
        player.set_parent(party);
    }
}
