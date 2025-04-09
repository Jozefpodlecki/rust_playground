use std::process::Command;
use log::*;
use tokio::signal;
use crate::{client::Client, models::{CommandArgs, ProcessType}, server::Server};
use anyhow::Result;

pub struct Controller {
}

impl Controller {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&self, args: &CommandArgs) -> Result<()> {
        match args.r#type {
            ProcessType::Server => {
                debug!("Running in SERVER mode");
    
                let port = args.port;
                let exe_path = std::env::current_exe().expect("Can't get current exe");
                let child_args = ["--type", "child", "--port", &port.to_string()];
                let mut child = Command::new(exe_path)
                    .args(child_args)
                    .spawn()
                    .expect("Failed to spawn child process");
    
                let server = Server::new();
                server.run(port).await?;

                tokio::select! {
                    result = server.run(port) => {
                        if let Err(e) = result {
                            error!("Server error: {}", e);
                        }
                    }
                    _ = signal::ctrl_c() => {
                        info!("Server: Shutdown signal received. Cleaning up...");
                    }
                }

                debug!("Waiting for child to finish...");
                child.wait()?;
            }
    
            ProcessType::Child => {
                let port = args.port;
                debug!("Running in CHILD mode on port {}", port);
                let client = Client::new();

                tokio::select! {
                    result = client.run(port) => {
                        if let Err(e) = result {
                            error!("Client error: {}", e);
                        }
                    }
                    _ = signal::ctrl_c() => {
                        info!("Client: Shutdown signal received. Cleaning up...");
                    }
                }
            }
        }

        Ok(())
    }
}