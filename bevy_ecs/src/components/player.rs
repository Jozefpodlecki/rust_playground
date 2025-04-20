use bevy_ecs::prelude::*;

#[derive(Component)]
pub struct Player(pub i32);

#[derive(Component)]
pub enum Class {
    Berserker,
    Bard,
    Aeromancer,
    Sorceress
}

#[derive(Component)]
pub struct Support;

#[derive(Component)]
pub struct DamageDealer;

#[derive(Component)]
pub struct AttackPower(pub u64);

#[derive(Component)]
pub struct Swiftness(pub u64);

#[derive(Component)]
pub struct CritRate(pub f64);
