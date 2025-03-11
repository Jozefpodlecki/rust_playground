
use anyhow::Result;
use simple_logger::SimpleLogger;
use wrapper::WindivertWrapper;

mod wrapper;

#[tokio::main]
async fn main() -> Result<()> {
    
    SimpleLogger::new().env().init().unwrap();

    let ip = "127.0.0.1"; 
    // let port = 6041;
    let port = 80;
    let mut windivert = WindivertWrapper::new(ip, port)?;

    windivert.start();

    Ok(())
}