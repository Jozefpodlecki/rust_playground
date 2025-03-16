use anyhow::Result;
use procedural_macros::AllVariants;
use simple_logger::SimpleLogger;

#[derive(AllVariants, Debug, Clone)]
pub enum Test {
    A,
    B,
    C
}

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    Test::all_variants();

    Ok(())
}