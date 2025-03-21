#[allow(dead_code)]

mod utils;
mod formatting;
use std::{thread::sleep, time::Duration};

use utils::{random_alphabetic_string, random_number_in_range};

fn main() {

    loop {
        let str = random_alphabetic_string(20);
        let random_integer = random_number_in_range(1..100);
        let random_float = random_number_in_range(1.0..100.0);
        println!("{} {} {}", str, random_integer, random_float);
        sleep(Duration::from_secs(1));
    }
}
