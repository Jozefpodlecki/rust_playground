#![allow(warnings)]

use std::time::Duration;

use bevy::{app::{App, ScheduleRunnerPlugin, Startup, Update}, time::{Time, TimePlugin}, MinimalPlugins};
use bevy_ecs::prelude::*;
mod components;
mod systems;
mod utils;
use systems::*;

fn main() {

    App::new()
        .add_plugins(TimePlugin)
        .add_plugins(ScheduleRunnerPlugin::run_loop(Duration::from_millis(500)))
        .add_systems(Startup, (setup))
        .add_systems(Update, (
            print_encounter,
            apply_buffs,
            reassign_targets_system,
            skill_casting_system,
            phase_trigger_system
        ))
        .run();

}
