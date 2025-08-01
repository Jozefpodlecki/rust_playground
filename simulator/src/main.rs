#![allow(warnings)]

use anyhow::*;
use flexi_logger::Logger;

use crate::core::{template::template_3dps_1support, Simulator};

mod core;

#[tokio::main]
async fn main() -> Result<()> {
    Logger::try_with_str("info")?.start()?;

    let template = template_3dps_1support();
    
    let simulator = Simulator::new(template);

    simulator.run();

    Ok(())
}
