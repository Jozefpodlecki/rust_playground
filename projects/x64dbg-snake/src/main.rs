#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(static_mut_refs)]
#![feature(pointer_is_aligned_to)]

use core::panic::PanicInfo;

use toolkit::println;

use crate::{ai::Algorithm, delay::sleep_ms, game::Game};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info:?}");
    loop {}
}

extern crate builtins;

mod ai;
mod arena;
mod game;
mod random;
mod snake;
mod delay;


#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let ai = ai::create_ai(Algorithm::Hybrid);
    // let ai = ai::create_ai(Algorithm::Greedy);
    // let ai = ai::create_ai(Algorithm::Bfs);
    let mut game = game::Game::new(ai);
    game.render();

    println!("Arena: {:p}", game.as_ptr());
    println!("Aligned: {}", game.as_ptr().is_aligned_to(4096));

    loop {
        game.tick();

        if game.game_over {
            println!("Game Over! Score: {}", game.score.get());
            break;
        }

        delay::sleep_ms(100);
    }

    loop {}
}