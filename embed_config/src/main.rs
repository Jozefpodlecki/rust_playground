use std::{env, path};

use dotenvy::dotenv;

fn main() {
    
    let api_key = match cfg!(debug_assertions) {
        true => {
            dotenv().ok();
            env::var("API_KEY").unwrap()
        },
        false => {
            option_env!("API_KEY").unwrap().to_string()
        },
    };

    println!("{}", api_key);
}
