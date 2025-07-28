use anyhow::*;

use crate::core::{template::template_3dps_1support, Simulator};

mod core;

#[tokio::main]
async fn main() -> Result<()> {
    flexi_logger::Logger::try_with_env().unwrap().start().unwrap();

    let template = template_3dps_1support();
    
    let simulator = Simulator::new(template);

    simulator.run();

    Ok(())
}
