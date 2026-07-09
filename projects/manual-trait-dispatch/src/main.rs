#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![feature(core_intrinsics)]
#![allow(unused)]
#![allow(internal_features)]
#![allow(improper_ctypes_definitions)] 
#![allow(invalid_reference_casting)]
#![feature(ptr_metadata)]

extern crate builtins;

mod oper_trait;
mod wrapper;

use core::{arch::naked_asm, intrinsics::black_box, marker::PhantomData, panic::PanicInfo, ptr::{DynMetadata, Pointee, metadata}};

use toolkit::{Sleeper, U8CStackString, println, rand::{self, Rng}};

use crate::{oper_trait::*, wrapper::TraitObjectWrapper};

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

fn print(args: &mut OperationArgs) -> CustomResult<OperationResult, u32> {
    println!("{}", args.name);
    CustomResult::Ok(OperationResult::Ignore)
}

fn generate(args: &mut OperationArgs) -> CustomResult<OperationResult, u32> {
    CustomResult::Ok(OperationResult::Generate(OperationData { value: [0; 10] }))
}

fn mutate(args: &mut OperationArgs) -> CustomResult<OperationResult, u32> {
    args.value[0] = 1;
    args.name = args.rng.rand_str_alpha::<10>();
    CustomResult::Ok(OperationResult::Mutate)
}

#[inline(never)]
pub fn get_handler(rng: &mut Rng) -> impl Operation {
    match rng.range_u32(0..3) {
        0 => generate,
        1 => mutate,
        _ => print
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    let mut value = [0; 10];
    let mut rng = Rng::new();
    let name = rng.rand_str_alpha::<10>();
    println!("name {}", name);
    println!("name {}", name.to_hex());
    let mut args = OperationArgs {
        rng,
        name,
        value: &mut value
    };

    let mut rng = Rng::new();
    
    loop {
        let handler = get_handler(&mut rng);
        let trait_ptr: *const dyn Operation = &handler as *const dyn Operation;
        let wrapper = TraitObjectWrapper::from_trait_ptr(trait_ptr);
        let result: CustomResult<OperationResult, u32> = wrapper.execute(&mut args);
        println!("name {}", args.name);
        println!("name {}", args.name.to_hex());
        println!("Result: {:?}", result);
        
        Sleeper::sleep(1000);
    }
    
    0
}
