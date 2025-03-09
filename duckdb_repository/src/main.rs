use orchestrator::Orchestrator;

mod db;
mod models;
mod orchestrator;

fn main() {

    let mut orchestrator = Orchestrator::new();

    match orchestrator.run() {
        Err(err) => println!("Error: {}", err),
        _ => {},
    }
}
