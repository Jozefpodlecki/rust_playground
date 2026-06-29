#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::panic::PanicInfo;

use utils::{Sleeper, println};

use crate::rand::Rng;

#[inline(never)]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

mod rand;
mod chacha;

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    
    
    loop {
        let mut rng = Rng::new();
    
        let x = rng.range_u32(1..10);
        let y = rng.range_u64(1, 100);
        let z = rng.range_f32(0.0, 1.0);
        let w = rng.range_f64(-10.0, 10.0);
        let b = rng.next_bool();
        
        println!( "u32: {}, u64: {}, f32: {:.6}, f64: {:.6}, bool: {}\n", x, y, z, w, b);
        Sleeper::sleep(1000);
    }

    0
}