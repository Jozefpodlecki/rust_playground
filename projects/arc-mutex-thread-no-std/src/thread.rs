use core::cell::UnsafeCell;
use core::mem;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use alloc::boxed::Box;
use ntapi::ntapi_base::CLIENT_ID;
use ntapi::ntmmapi::NtAllocateVirtualMemory;
use ntapi::ntmmapi::NtProtectVirtualMemory;
use ntapi::ntobapi::NtClose;
use ntapi::ntobapi::NtWaitForSingleObject;
use ntapi::ntpsapi::INITIAL_TEB;
use ntapi::ntpsapi::INITIAL_TEB_OldInitialTeb;
use ntapi::ntpsapi::NtCreateThread;
use ntapi::ntpsapi::NtCreateThreadEx;
use ntapi::ntpsapi::NtCurrentProcess;
use ntapi::ntrtl::RtlExitUserThread;
use utils::NtDll;
use utils::println;
use winapi::shared::ntdef::HANDLE;
use winapi::shared::ntdef::NTSTATUS;
use winapi::shared::ntdef::PVOID;
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::winnt::CONTEXT;
use winapi::um::winnt::LARGE_INTEGER;
use winapi::um::winnt::MEM_COMMIT;
use winapi::um::winnt::MEM_RELEASE;
use winapi::um::winnt::MEM_RESERVE;
use winapi::um::winnt::PAGE_GUARD;
use winapi::um::winnt::PAGE_READWRITE;
use winapi::um::winnt::THREAD_ALL_ACCESS;
use crate::arc::Arc;

#[repr(align(16))]
struct AlignedContext(CONTEXT);

struct PacketInner<T> {
    finished: AtomicBool,
    result: UnsafeCell<Option<T>>,
    func: UnsafeCell<Option<Box<dyn FnOnce() -> T + Send>>>,
}

pub struct Packet<T>(Arc<PacketInner<T>>);

impl<T> Clone for Packet<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> Packet<T> {
    pub fn new(func: impl FnOnce() -> T + Send + 'static) -> Self {
        Self(Arc::new(PacketInner {
            finished: AtomicBool::new(false),
            result: UnsafeCell::new(None),
            func: UnsafeCell::new(Some(Box::new(func))),
        }))
    }

    pub fn from_ptr(ptr: *mut Self) -> Self {
        unsafe { ptr.read() }
    }

    pub fn execute(&self) -> T {
        unsafe {
            let inner = &*self.0;
            let func_ptr = &mut *inner.func.get();
            let func = func_ptr.take().unwrap();
            func()
        }
    }

    pub fn set_result(&self, result: T) {
        unsafe {
            let inner = &*self.0;
            inner.result.get().write(Some(result));
            inner.finished.store(true, Ordering::Release);
        }
    }

    pub fn is_finished(&self) -> bool {
        (*self.0).finished.load(Ordering::Acquire)
    }

    pub fn take_result(&self) -> Option<T> {
        unsafe {
            let inner = &*self.0;
            (*inner.result.get()).take()
        }
    }

    pub fn leak(&self) -> *mut Self {
        let leaked: &mut Self = Box::leak(Box::new(self.clone()));
        leaked as *mut Self
    }
}

pub struct JoinHandle<T> {
    handle: HANDLE,
    packet: Packet<T>,
}

impl<T> JoinHandle<T> {
    pub fn is_finished(&self) -> bool {
        
        let status = unsafe {
            let mut delay: LARGE_INTEGER = core::mem::zeroed();
            *delay.QuadPart_mut() = 0;
            NtWaitForSingleObject(
                self.handle,
                0,
                &mut delay,
            )
        };
        status == STATUS_SUCCESS
    }

    pub fn join(self) -> Result<T, ()> {
        unsafe {
            let status = NtWaitForSingleObject(
                self.handle,
                0,
                ptr::null_mut(),
            );
            
            if status != STATUS_SUCCESS {
                return Err(());
            }
            
            NtClose(self.handle);
            
            self.packet.take_result().ok_or(())
        }
    }
}

pub struct Thread;

impl Thread {
    const STACK_SIZE: usize = 1024 * 1024; // 1MB default stack
    const STACK_COMMIT: usize = 64 * 1024; // 64KB initial commit
    
    pub fn spawn<F, T>(func: F) -> Result<JoinHandle<T>, NTSTATUS>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let packet = Packet::new(func);

        unsafe extern "system" fn thread_entry<T>(param: *mut winapi::ctypes::c_void) {
            
            println!("\nthread_entry param \"{:p}\"", param);
            // let packet = Packet::<T>::from_ptr(param as _);
            // let result = packet.execute();
            // packet.set_result(result);
            
            // unsafe { RtlExitUserThread(0) };
        }

        let packet_clone = packet.leak();
        let param_ptr: *mut winapi::ctypes::c_void = packet_clone as _;
        let mut thread_handle: HANDLE = ptr::null_mut();
        let mut context = AlignedContext(unsafe { core::mem::zeroed() });
        let stack_allocation_base = Self::allocate_stack()?;
        // let stack_base = stack_allocation_base;
        // let stack_limit = unsafe { stack_base.add(Self::STACK_SIZE) };

        let stack_top = unsafe { 
            stack_allocation_base.add(Self::STACK_SIZE - (5 * 4096)) 
        };
        let stack_bottom = stack_allocation_base;  // Bottom of stack
        
        let mut initial_teb = INITIAL_TEB {
            OldInitialTeb: INITIAL_TEB_OldInitialTeb {
                OldStackBase: ptr::null_mut(),
                OldStackLimit: ptr::null_mut(),
            },
            // StackBase: stack_base as _,
            // StackLimit: stack_limit as _,
            StackBase: stack_top as _,
            StackLimit: stack_bottom as _,
            StackAllocationBase: stack_allocation_base as _,
        };

        let ntdll = NtDll::from_current_process();
// ntapi::ntrtl::RtlUserThreadStart
        println!("NtDll base {:p}", ntdll.base());
println!("RtlUserThreadStart {:p}", ntdll.RtlUserThreadStart());
println!("thread_entry {:p}", thread_entry::<T> as PVOID);
println!("param_ptr {:p}", param_ptr);
        context.0.ContextFlags = 0x00100000;
        context.0.SegCs = 0x1b;  // 64-bit code segment
        context.0.SegDs = 0x20;  // Data segment
        context.0.SegEs = 0x20;  // Data segment
        context.0.SegFs = 0x38;  // TEB segment (important!)
        context.0.SegGs = 0;     // GS not used in user mode
        context.0.SegSs = 0x20;  // Stack segment
        context.0.Rax = 0;
        context.0.Rdx = 0;
        context.0.R8 = 0;
        context.0.R9 = 0;
        
        context.0.Rip = ntdll.RtlUserThreadStart() as _;
        context.0.Rcx = thread_entry::<T> as _;
        context.0.Rdx = param_ptr as u64;

        // context.0.Rip = thread_entry::<T> as _;
        // context.0.Rcx = param_ptr as u64;
        // context.0.Rdx = param_ptr as u64;

        context.0.Rsp = ((stack_top as usize) & !0xF) as u64;
        context.0.Rbp = 0;
        context.0.Rsi = 0;
        context.0.Rdi = 0;
        context.0.Rbx = 0;
        let mut client_id: CLIENT_ID = unsafe { mem::zeroed() };

        let status = unsafe {
            NtCreateThread(
                &mut thread_handle as _,
                THREAD_ALL_ACCESS,
                ptr::null_mut(),
                NtCurrentProcess,
                &mut client_id as _,
                &mut context.0 as _,
                &mut initial_teb as _,
                0,
            )
        };
        
        println!("NtCreateThread \"{status}\"");
        
        if status != STATUS_SUCCESS {
            println!("test");
            return Err(status);
        }

        Ok(JoinHandle {
            handle: thread_handle,
            packet,
        })
    }

    fn allocate_stack() -> Result<*mut u8, NTSTATUS> {
        unsafe {
            let stack_size = Self::STACK_SIZE;
            let mut base: PVOID = ptr::null_mut();
            let mut region_size = stack_size as usize;

            let status = NtAllocateVirtualMemory(
                NtCurrentProcess,
                &mut base,
                0,
                &mut region_size,
                MEM_RESERVE,
                PAGE_READWRITE,
            );

            if status != STATUS_SUCCESS {
                println!("NtAllocateVirtualMemory");
                return Err(status);
            }


            let commit_size = stack_size - 4096;
            let mut commit_base = base;
            let mut commit_region_size = commit_size;
            
            let status = NtAllocateVirtualMemory(
                NtCurrentProcess,
                &mut commit_base,
                0,
                &mut commit_region_size,
                MEM_COMMIT,
                PAGE_READWRITE,
            );

            if status != STATUS_SUCCESS {
                println!("NtAllocateVirtualMemory");
                let _ = NtAllocateVirtualMemory(
                    NtCurrentProcess,
                    &mut base,
                    0,
                    &mut (stack_size as usize),
                    MEM_RELEASE,
                    PAGE_READWRITE,
                );
                return Err(status);
            }

            
            let mut guard_page_addr = base.add(stack_size - 4096);
            let mut guard_page_size = 4096;
            
            let status = NtAllocateVirtualMemory(
                NtCurrentProcess,
                &mut guard_page_addr,
                0,
                &mut guard_page_size,
                MEM_COMMIT,
                PAGE_READWRITE | PAGE_GUARD,
            );

            if status != STATUS_SUCCESS {
                println!("NtAllocateVirtualMemory");
                let _ = NtAllocateVirtualMemory(
                    NtCurrentProcess,
                    &mut base,
                    0,
                    &mut (stack_size as usize),
                    MEM_RELEASE,
                    PAGE_READWRITE,
                );
                return Err(status);
            }

            Ok(base as *mut u8)
        }
    }

    pub fn spawn_ex<F, T>(func: F) -> Result<JoinHandle<T>, NTSTATUS>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let packet = Packet::new(func);
        
        unsafe extern "system" fn thread_entry<T>(param: *mut winapi::ctypes::c_void) -> u32 {
            let packet = Packet::<T>::from_ptr(param as _);
            let result = packet.execute();
            packet.set_result(result);
            
            0
        }

        let packet_clone = packet.leak();
        let param_ptr: *mut winapi::ctypes::c_void = packet_clone as _;
        let mut thread_handle: HANDLE = ptr::null_mut();

        let status = unsafe {
            NtCreateThreadEx(
                &mut thread_handle,
                THREAD_ALL_ACCESS,
                ptr::null_mut(),
                NtCurrentProcess,
                thread_entry::<T> as *mut _,
                param_ptr,
                0,
                0,
                0,
                0,
                ptr::null_mut(),
            )
        };

        if status != STATUS_SUCCESS {
            return Err(status);
        }

        Ok(JoinHandle {
            handle: thread_handle,
            packet,
        })
    }
}