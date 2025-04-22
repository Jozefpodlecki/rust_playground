#![allow(warnings)]

use std::time::Duration;

use bevy_ecs::prelude::*;
use models::EncounterTemplate;
mod models;

fn main() {

    let json_bytes = include_bytes!("scenarios/behemoth_g1.json");

    let template: EncounterTemplate = serde_json::from_slice(json_bytes).unwrap();

    println!("{template:#?}");
}
