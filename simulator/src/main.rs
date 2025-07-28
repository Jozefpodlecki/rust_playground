use anyhow::*;

use crate::core::{template::template_3dps_1support, Simulator};

mod core;

#[tokio::main]
async fn main() -> Result<()> {

    let template = template_3dps_1support();
    
    let simulator = Simulator::new(template);

    simulator.run();

    Ok(())
}
