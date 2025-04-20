
use bevy::utils::HashMap;
use bevy_ecs::prelude::*;
use std::collections::HashSet;
use std::{io::Stdout, time::{Duration, Instant}};

#[derive(Component)]
pub struct Health {
    pub current: u64,
    pub max: u64,
    pub hp_percentage: f64,
    pub hp_max_bars: u64,
    pub hp_current_bars: u64,
}

impl Health {
    pub fn new(max_hp: u64, max_bars: u64) -> Self {
        let current = max_hp;
        let hp_percentage = 100.0;
        let hp_current_bars = max_bars;

        Health {
            current,
            max: max_hp,
            hp_percentage,
            hp_max_bars: max_bars,
            hp_current_bars,
        }
    }

    pub fn update_health_bars(&mut self) {
        self.hp_percentage = (self.current as f64 / self.max as f64) * 100.0;
        self.hp_current_bars = ((self.hp_percentage / 100.0) * self.hp_max_bars as f64).round() as u64;
    }
}