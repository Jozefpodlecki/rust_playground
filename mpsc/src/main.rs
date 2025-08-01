#![allow(warnings)]

use etherparse::{Ipv4Header, PacketBuilder, PacketHeaders, TcpHeader};
use flexi_logger::{Duplicate, Logger};
use log::*;
use rand::{distr::{Distribution, StandardUniform}, rng, Rng};
use windivert::{layer::NetworkLayer, packet::WinDivertPacket, prelude::WinDivertFlags, CloseAction, WinDivert};
use std::{net::Ipv4Addr, panic, process::exit, sync::{atomic::{AtomicU32, AtomicU8, Ordering}, mpsc}, thread::{sleep, spawn}, time::{Duration, Instant}, vec};

pub enum Action {
    Add,
    Remove,
    Half
}

impl Distribution<Action> for StandardUniform {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Action {
        match rng.random_range(0..=5) {
            0 => Action::Add,
            1 => Action::Add,
            2 => Action::Add,
            3 => Action::Remove,
            4 => Action::Remove,
            _ => Action::Half,
        }
    }
}

#[unsafe(no_mangle)]
pub static TARGET_BYTE: AtomicU32 = AtomicU32::new(0x75BCD15);

fn main() {
    Logger::try_with_str("debug").unwrap()
        .duplicate_to_stderr(Duplicate::Warn)
        .start()
        .unwrap();


    let (tx, rx) = mpsc::channel::<(Action, Vec<u8>)>();

    spawn(move || {
        let mut rng = rng();
        let now = Instant::now();

        loop {
            sleep(Duration::from_secs(5));

            if now.elapsed() > Duration::from_secs(60) {
                info!("Exiting");
                exit(0)
            }
            
            TARGET_BYTE.store(69, Ordering::SeqCst);

            let action: Action = rng.random();
            let size = rng.random_range(1..=10);
            let data: Vec<u8> = (0..size).map(|_| rng.random()).collect();

            tx.send((action, data)).unwrap();
            sleep(Duration::from_secs(5));
        }
    });

    loop {
        let (action, data) = rx.recv().unwrap();
        let mut value = 0;

        match action {
            Action::Add => {
                value += data.iter().copied().map(i32::from).sum::<i32>();
            },
            Action::Remove => {
                value -= data.iter().copied().map(i32::from).sum::<i32>();
            },
            Action::Half => {
                value = value / 2;
            },
        }
    }
}