use core::{mem::{self, zeroed}, ptr::null_mut};
use toolkit::{Sleeper, println};
use winapi::{shared::{minwindef::LPVOID, ntdef::PVOID}, um::{fibersapi::IsThreadAFiber, winbase::*}};

#[derive(Default, Clone, Copy)]
pub struct FiberId(pub u32);

#[derive(Default, Clone, Copy)]
pub struct FiberCounter(pub u32);

#[derive(Default, Clone, Copy)]
pub struct FiberArgs {
    pub id: FiberId,
    pub parent: LPVOID,
    pub counter: FiberCounter,
}

#[derive(Default, Clone, Copy)]
pub struct Fiber(LPVOID);

impl Fiber {
    
    pub fn switch(&self) {
        unsafe {
            SwitchToFiber(self.0);
        }
    }
}

pub struct FiberScheduler {
    fibers: [Option<Fiber>; 10],
    args: [FiberArgs; 10],
    current: usize,
}

pub fn current_fiber() -> PVOID {
    let fiber_ptr: PVOID;
    unsafe { core::arch::asm!("mov rax, qword ptr fs:[0x10]", out("rax") fiber_ptr) };
    fiber_ptr
}

static mut SCHEDULER: FiberScheduler = FiberScheduler {
    fibers: [None; 10],
    args: [unsafe { zeroed() }; 10],
    current: 0,
};

impl FiberScheduler {
    pub fn new() -> Option<&'static mut Self> {
        unsafe {
            let main = if IsThreadAFiber() != 0 {
                current_fiber()
            } else {
                let fiber = ConvertThreadToFiber(null_mut());
                if fiber.is_null() {
                    return None;
                }
                fiber
            };
            
            let mut scheduler = &mut SCHEDULER;
            
            for i in 0..10 {
                scheduler.args[i].id = FiberId(i as u32);
                scheduler.args[i].parent = main;
                
                let ptr = CreateFiberEx(
                    0,
                    0,
                    0,
                    Some(fiber_entry as _),
                    &mut scheduler.args[i] as *mut _ as LPVOID,
                );
                
                if !ptr.is_null() {
                    scheduler.fibers[i] = Some(Fiber(ptr));
                }
            }
            
            Some(scheduler)
        }
    }
    
    pub fn run(&mut self) {
        loop {
            for i in 0..10 {
                if let Some(fiber) = &self.fibers[i] {
                    fiber.switch();
                    Sleeper::sleep(100);
                }
            }
        }
    }
}

extern "system" fn fiber_entry(param: LPVOID) {
    
    unsafe {
        let args_ptr = param as *mut FiberArgs;
        let args = &mut *args_ptr;
        println!("Fiber id: {}", args.id.0);

        loop {
            args.counter.0 += 1;
            println!("Fiber id: {}, counter: {}", args.id.0, args.counter.0);
            Sleeper::sleep(100);
            SwitchToFiber(args.parent);
        }
    }
}