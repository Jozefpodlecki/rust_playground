use core::{mem::zeroed, ptr::{self, null_mut}};

use ntapi::{ntexapi::SYSTEM_THREAD_INFORMATION, ntobapi::NtWaitForSingleObject, ntpsapi::{NtGetContextThread, THREAD_BASIC_INFORMATION, ThreadBasicInformation, ThreadSuspendCount, ThreadSystemThreadInformation}};
use toolkit::{AlignedContext, ProcessMemoryReader, println, syscalls::NtQueryInformationThread};
use winapi::um::winnt::{CONTEXT, CONTEXT_FULL, LARGE_INTEGER};

pub enum ThreadWaitReason {
    Executive = 0,
    FreePage = 1,
    PageIn = 2,
    SystemAllocation = 3,
    ExecutionDelay = 4,
    Suspended = 5,
    UserRequest = 6,
    EventPairHigh = 7,
    EventPairLow = 8,
    LpcReceive = 9,
    LpcReply = 10,
    VirtualMemory = 11,
    PageOut = 12,
    Unknown = 13
}

pub fn check_remote_thread(
    process_handle: *mut winapi::ctypes::c_void, 
    thread_handle: *mut winapi::ctypes::c_void,
    routine_addr: *mut winapi::ctypes::c_void) {
    unsafe {
        let mut delay: LARGE_INTEGER = zeroed();
        *delay.QuadPart_mut() = -50_000_000; // 5 seconds in 100ns units

        let status = NtWaitForSingleObject(
            thread_handle,
            0,
            &mut delay,
        );
        println!("NtWaitForSingleObject: 0x{:X}", status);

        let mut context: AlignedContext = AlignedContext::default();
        context.ContextFlags = CONTEXT_FULL;

        let status = NtGetContextThread(
            thread_handle,
            &mut *context,
        );
        println!("NtGetContextThread: 0x{:X}", status);

        let mut info: THREAD_BASIC_INFORMATION = zeroed();
        let status = ntapi::ntpsapi::NtQueryInformationThread(
            thread_handle,
            ThreadBasicInformation, 
            &mut info as *mut _ as *mut _,
            size_of::<THREAD_BASIC_INFORMATION>() as u32,
            null_mut(),
        );
        println!("NtQueryInformationThread: 0x{:X}", status);

        if status >= 0 {
            println!("ExitStatus: 0x{:X}", info.ExitStatus);
            println!("TebBaseAddress: {:p}", info.TebBaseAddress);
            println!("ClientId.UniqueProcess: {:?}", info.ClientId.UniqueProcess);
            println!("ClientId.UniqueThread: {:?}", info.ClientId.UniqueThread);
            println!("AffinityMask: 0x{:X}", info.AffinityMask);
            println!("Priority: {}", info.Priority);
            println!("BasePriority: {}", info.BasePriority);
        }

        let mut suspend_count: u32 = 0;
        let status = NtQueryInformationThread(
            thread_handle,
            ThreadSuspendCount, // 
            &mut suspend_count as *mut _ as *mut _,
            size_of::<u32>() as u32,
            null_mut(),
        );

        if status >= 0 {
            println!("SuspendCount: {}", suspend_count);
        }

        let mut info: SYSTEM_THREAD_INFORMATION = zeroed();
        let status = NtQueryInformationThread(
            thread_handle,
            ThreadSystemThreadInformation,
            &mut info as *mut _ as *mut _,
            size_of::<SYSTEM_THREAD_INFORMATION>() as u32,
            null_mut(),
        );
        println!("NtQueryInformationThread: 0x{:X}", status);

        if status >= 0 {
            println!("ThreadState: {}", info.ThreadState);
            println!("WaitReason: {}", info.WaitReason);
        }

        if status >= 0 {
            println!("RIP: 0x{:X}", context.Rip);
            println!("RSP: 0x{:X}", context.Rsp);
            println!("RAX: 0x{:X}", context.Rax);
            println!("RIP - routine: 0x{:X}", context.Rip - routine_addr as u64);

            let bytes = ProcessMemoryReader::read_remote_bytes_fixed::<200>(process_handle, context.Rip as _).unwrap();
            println!("{}", bytes);

            if context.Rip == routine_addr as u64 {
                println!("✅ Thread is executing at the routine!");
            } else if context.Rip == routine_addr as u64 + 2 {
                println!("✅ Thread executed the routine and moved on!");
            } else {
                println!("⚠️ Thread is executing at 0x{:X} (not at the routine)", context.Rip);
            }
        }
    }
}