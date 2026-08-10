#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::{panic::PanicInfo, ptr::null_mut};

use toolkit::{FreeListAllocator, ProcessSpawner, Sleeper, Thread, U16CStackString, println};

use crate::event::EventHandle;

mod event;

extern crate builtins;

#[macro_use]
extern crate alloc;

#[global_allocator]
static ALLOCATOR: FreeListAllocator<8192> = FreeListAllocator::new();

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    let name = U16CStackString::<50>::from_str("custom_event").unwrap();
    let event_handle = EventHandle::manual(&name).unwrap();

    let handle1 = Thread::spawn(move || {
        let event_handle = event_handle;

        event_handle.wait();
        println!("1");
    }).unwrap();

    let handle2 = Thread::spawn(move || {
        let event_handle = event_handle;

        event_handle.wait();
        Sleeper::sleep(100);
        println!("2");
    }).unwrap();

    Sleeper::sleep(1000);
    event_handle.set();
    event_handle.reset();

    handle1.join().unwrap();
    handle2.join().unwrap();
    Sleeper::sleep(1000);
   
    0
}