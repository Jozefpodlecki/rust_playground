use core::cell::UnsafeCell;
use core::mem;
use core::ptr;
use core::sync::atomic::{AtomicBool, Ordering};
use alloc::boxed::Box;
use ntapi::ntapi_base::CLIENT_ID;
use ntapi::ntobapi::*;
use ntapi::ntpsapi::*;
use ntapi::ntpsapi::NtCurrentProcess;
use ntapi::ntrtl::RtlExitUserThread;
use toolkit::*;
use winapi::shared::ntdef::HANDLE;
use winapi::shared::ntdef::NTSTATUS;
use winapi::shared::ntstatus::STATUS_SUCCESS;
use winapi::um::winnt::*;

// struct PacketInner<T> {
//     finished: AtomicBool,
//     result: UnsafeCell<Option<T>>,
//     func: UnsafeCell<Option<Box<dyn FnOnce() -> T + Send>>>,
// }

// pub struct Packet<T>(Arc<PacketInner<T>>);

// impl<T> Clone for Packet<T> {
//     fn clone(&self) -> Self {
//         Self(self.0.clone())
//     }
// }

// impl<T> Packet<T> {
//     pub fn new(func: impl FnOnce() -> T + Send + 'static) -> Self {
//         Self(Arc::new(PacketInner {
//             finished: AtomicBool::new(false),
//             result: UnsafeCell::new(None),
//             func: UnsafeCell::new(Some(Box::new(func))),
//         }))
//     }

//     pub fn from_ptr(ptr: *mut Self) -> Self {
//         // let inner_ptr = ptr as *mut Packet<T>;
//         // unsafe { Self(Arc::from_raw_arcinner(inner_ptr)) }
//         unsafe { ptr.read() }
//     }

//     pub fn execute(&self) -> T {
//         unsafe {
//             let inner = &*self.0;
//             let func_ptr = &mut *inner.func.get();
//             let func = func_ptr.take().unwrap();
//             func()
//         }
//     }

//     pub fn set_result(&self, result: T) {
//         unsafe {
//             let inner = &*self.0;
//             inner.result.get().write(Some(result));
//             inner.finished.store(true, Ordering::Release);
//         }
//     }

//     pub fn is_finished(&self) -> bool {
//         (*self.0).finished.load(Ordering::Acquire)
//     }

//     pub fn take_result(&self) -> Option<T> {
//         unsafe {
//             let inner = &*self.0;
//             (*inner.result.get()).take()
//         }
//     }

//     pub fn into_raw(&self) -> *mut Self {
//         let leaked: &mut Self = Box::leak(Box::new(self.clone()));
//         leaked as *mut Self

//         // let cloned = self.clone();
//         // let ptr = Arc::as_ptr(&cloned.0) as *const Self as *mut Self;
//         // core::mem::forget(cloned);
//         // ptr
//     }
// }

pub struct JoinHandle<T> {
    handle: HANDLE,
    packet: Packet<T>,
    stack: ThreadStack
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

    pub fn join(self) -> Result<T, NTSTATUS> {
        unsafe {
            let status = NtWaitForSingleObject(
                self.handle,
                0,
                ptr::null_mut(),
            );
            println!("NtWaitForSingleObject 0x{status:X}");
            if status != STATUS_SUCCESS {
                return Err(status);
            }
            
            NtClose(self.handle);
            
            self.stack.free().unwrap();
            self.packet.take_result().ok_or(99999)
        }
    }
}

pub struct Thread;

impl Thread {
    
    pub fn spawn<F, T>(func: F) -> Result<JoinHandle<T>, NTSTATUS>
    where
        F: FnOnce() -> T,
        F: Send + 'static,
        T: Send + 'static,
    {
        let packet = Packet::new(func);

        unsafe extern "system" fn thread_entry<T>(param: *mut winapi::ctypes::c_void) {
            
            let packet = Packet::<T>::from_ptr(param as *mut Packet<T>);
            let result = packet.execute();
            packet.set_result(result);
            
            unsafe { RtlExitUserThread(0) };
        }

        let packet_clone = packet.into_raw();
        let param_ptr: *mut winapi::ctypes::c_void = packet_clone as _;
        let mut thread_handle: HANDLE = ptr::null_mut();
        let mut context = ThreadContext::new();
        let stack_size = 1024 * 1024;
        let stack = ThreadStack::new(stack_size).unwrap();
        let mut initial_teb = stack.to_initial_teb();
        let rsp = stack.rsp();

        let rip = thread_entry::<T> as _;
        let rcx = param_ptr as u64;
        context.set_thread_context(rsp, rip, rcx);

        // context.0.Rip = ntdll.RtlUserThreadStart() as _;
        // context.0.Rcx = 0;
        // context.0.Rdx = thread_entry::<T> as _;
        // context.0.R8 = param_ptr as u64;
        
        // context.0.Rip = ntdll.RtlUserThreadStart() as _;
        // context.0.Rcx = thread_entry::<T> as _;
        // context.0.Rdx = param_ptr as u64;

        let mut client_id: CLIENT_ID = unsafe { mem::zeroed() };

        let status = unsafe {
            NtCreateThread(
                &mut thread_handle as _,
                THREAD_ALL_ACCESS,
                ptr::null_mut(),
                NtCurrentProcess,
                &mut client_id as _,
                &mut *context,
                &mut initial_teb as _,
                0,
            )
        };
        
        if status != STATUS_SUCCESS {
            stack.free().unwrap();
            return Err(status);
        }

        Ok(JoinHandle {
            handle: thread_handle,
            packet,
            stack
        })
    }
}