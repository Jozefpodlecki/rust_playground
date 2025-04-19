pub mod sorceress;
pub mod aeromancer;
pub mod paladin;
pub mod artist;
pub mod artillerist;
pub mod bard;
pub mod generic;
pub mod builder;

pub use builder::PlayerTemplateBuilder;

use chrono::{DateTime, Duration, Utc};

use super::Class;