use ntapi::ntpsapi::NtCurrentThreadId ;
use toolkit::{JoinHandle, ProcessMemoryReader, SuspendedThread, SystemError, Thread, ThreadHandle, ThreadOpenFlags, println, syscalls::{NtQueryInformationThread, NtReadVirtualMemory}};
use winapi::{shared::{ntdef::HANDLE, ntstatus::STATUS_SUCCESS}, um::winnt::{LARGE_INTEGER, MEM_COMMIT, MEM_RESERVE, PAGE_READWRITE}};

use crate::{utils::*};

pub fn case_alertable_infinite() -> Result<(), SystemError>  {
    let main_thread = ThreadHandle::open_current(ThreadOpenFlags::ALL)?;

    let handle: JoinHandle<Result<(), SystemError>> = Thread::spawn_ex(move || {
        let main_thread = main_thread;
        delay_secs(1, 2);
        main_thread.alert_resume()?;

        Ok(())
    }).map_err(SystemError::Thread)?;

    delay_infinite(1);

    handle.join().map_err(SystemError::Thread)?;

    Ok(())
}

pub fn case_not_alertable_infinite() -> Result<(), SystemError> {
    let main_thread = ThreadHandle::open_current(ThreadOpenFlags::ALL)?;

    let handle = Thread::spawn_ex(move || {
        let main_thread = main_thread;

        unsafe {

            let run = || {
                let suspended = SuspendedThread::new().map_err(SystemError::Thread)?;
                let suspended_tid = suspended.tid().map_err(SystemError::Thread)?;
                let suspended_thread = ThreadHandle::open(suspended_tid as _, ThreadOpenFlags::ALL)?;
                
                let mut context = main_thread.get_context()?;

                let stack_info = copy_thread_stack(main_thread.handle())?;
                let rsp_offset = context.Rsp as usize - stack_info.original_stack_limit;
                let new_rsp = stack_info.new_base + rsp_offset;
                context.Rsp = new_rsp as u64;

                suspended_thread.set_context(context)?;
                main_thread.terminate(0)?;
                let updated_context = suspended_thread.get_context()?;

                suspended_thread.resume()?;
                

                Ok::<(), SystemError>(())
            };

            if let Err(err) = run() {
                println!("{err:?}");
            }
        }
    }).unwrap();

    delay_infinite(0);

    match handle.join() {
        Ok(_) => {
            println!("join");
        },
        Err(err) => {
            println!("{err}");
        },
    }

    Ok(())
}