#![allow(warnings)]

use anyhow::Result;
use flexi_logger::{Duplicate, Logger};
use log::info;
use iced_x86::{Decoder, DecoderOptions, Encoder, Instruction};

fn main() -> Result<()> {
    Logger::try_with_str("debug").unwrap()
        .duplicate_to_stderr(Duplicate::Warn)
        .start()
        .unwrap();

    let mut encoder = Encoder::new(64);
    // let mut value = vec![];

    let mut instruction = Instruction::with_rep_movsb(64)?;
    instruction.set_op_register(0, iced_x86::Register::RDI);
    instruction.set_op_register(1, iced_x86::Register::RSI);
    encoder.encode(&instruction, 0x0);

    let buffer = encoder.take_buffer();
    //  rep movsb byte ptr [rdi], byte ptr [rsi]
    info!("{:?}", buffer);

    let mut decoder = Decoder::new(64, &buffer, DecoderOptions::NONE);

    let instruction = decoder.decode();

    info!("{:?}", instruction.to_string());

    Ok(())
}