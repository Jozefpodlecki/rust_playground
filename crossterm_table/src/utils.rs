use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use rand::{distr::uniform::{SampleRange, SampleUniform}, rng, Rng};

use crate::models::*;

pub fn generate_separator(length: usize) -> String {
    "-".repeat(length) + "\n"
}

pub fn format_duration(duration_seconds: i64) -> String {
    let hours = duration_seconds / 3600;
    let minutes = (duration_seconds % 3600) / 60;
    let remaining_secs = duration_seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, remaining_secs)
}

pub fn format_unit(value: u64) -> String {
    match value {
        0..=999 => format!("{}", value),
        1_000..=999_999 => format!("{:.1}k", value as f64 / 1_000.0),
        1_000_000..=999_999_999 => format!("{:.1}m", value as f64 / 1_000_000.0),
        _ => format!("{:.1}b", value as f64 / 1_000_000_000.0),
    }
}


pub fn random_alphabetic_string_capitalized(length: usize) -> String {
    let charset: &[u8] = b"abcdefghijklmnopqrstuvwxyz";
    let mut rng = rng();
    
    let mut result: String = (0..length)
        .map(|_| charset[rng.random_range(0..charset.len())] as char)
        .collect();

    let first_char = result.get_mut(0..1).unwrap();
    first_char.make_ascii_uppercase();

    result
}

pub fn random_number_in_range<T: SampleUniform, R: SampleRange<T>>(range: R) -> T {
    let mut rng = rng();
    rng.random_range(range)
}