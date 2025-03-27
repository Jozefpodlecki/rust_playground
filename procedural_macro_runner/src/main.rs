use anyhow::Result;
use procedural_macros::{AllVariants, Service};
use simple_logger::SimpleLogger;

#[derive(AllVariants, Debug, Clone)]
pub enum Test {
    A,
    B,
    C
}

#[derive(Service, Debug, Clone)]
pub struct Something {

}

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    Test::all_variants();

    let service = Something {};
    println!("{}", service.name());

    Ok(())
}