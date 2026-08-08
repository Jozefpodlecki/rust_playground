#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::{arch::naked_asm, mem, panic::PanicInfo, ptr};
use toolkit::{ProcessEnvironmentBlock, U8CStackString, println, stack_trait::{Buf, Stacked}};

use crate::{client::Client, error::MainError, server::{Server, ServerOptions}};

extern crate builtins;

mod error;
mod utils;
mod client_info;
mod types;
mod shared;
mod server;
mod client;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}



pub fn run() -> Result<(), MainError> {
    let peb = ProcessEnvironmentBlock::current_process();
    let args = peb.command_line();

    if let Some(arg) = args.at(1) {
        let client = Client::new()?;
        client.run()?;
    }

    let options = ServerOptions {
        spawn_client: true
    };
    let mut server = Server::new(options)?;
    server.run()?;

    Ok(())
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> u32 {

    if let Err(err) = run() {
        println!("{}", err);
        return err.to_code();
    }

    0
}