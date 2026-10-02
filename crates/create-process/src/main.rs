#![no_std]
#![no_main]
#![windows_subsystem = "console"]

use core::panic::PanicInfo;
use core::ptr::null_mut;

use ntapi::winapi_local::um::winnt::NtCurrentTeb;
use toolkit::{ProcessSpawner, Sleeper, U16CStackString, UnicodeString, println};
extern crate compiler_builtins;
use winapi::{ctypes::c_void, shared::{basetsd::SIZE_T, minwindef::FALSE, ntdef::{OBJECT_ATTRIBUTES, PVOID, UNICODE_STRING}, ntstatus::STATUS_ACCESS_DENIED}, um::{processthreadsapi::{CreateProcessW, PROCESS_INFORMATION, STARTUPINFOW}, winbase::{CREATE_NO_WINDOW, CREATE_SUSPENDED, DETACHED_PROCESS}, winnt::{IMAGE_NT_SIGNATURE, LARGE_INTEGER, MEM_COMMIT, MEM_IMAGE, MEM_RESERVE, MEMORY_BASIC_INFORMATION, NT_TIB, PAGE_EXECUTE_READWRITE, PAGE_READONLY, PAGE_READWRITE, PROCESS_ALL_ACCESS, RtlCopyMemory, THREAD_ALL_ACCESS}}};


#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{info}");
    loop {}
}

pub fn get_last_error() -> u32 {
    unsafe {
        let teb = NtCurrentTeb();
        (*teb).LastErrorValue
    }
}

pub fn create_process(exe_path: &str, current_dir: Option<&str>, flags: u32) -> Result<PROCESS_INFORMATION, i32> {
    let mut startup_info: STARTUPINFOW = unsafe { core::mem::zeroed() };
    startup_info.cb = core::mem::size_of::<STARTUPINFOW>() as u32;
    let mut process_info: PROCESS_INFORMATION = unsafe { core::mem::zeroed() };

    let wide_path = exe_path
        .encode_utf16()
        .chain(core::iter::once(0))
        .collect::<heapless::Vec<u16,260>>();
  

    if unsafe {
        CreateProcessW(
            wide_path.as_ptr() as _,
            null_mut(),
            // null_mut(),
            // wide_path.as_ptr() as _,
            null_mut(),
            null_mut(),
            0,
            flags,
            null_mut(),
            null_mut(),
            &mut startup_info,
            &mut process_info,
        )
    } == 0 {
         
        let last_error = get_last_error();
        return Err(last_error as i32);
    }
 println!("{wide_path:?}");
    Ok(process_info)
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {

    // let path = r#"C:\repos\rust_playground\projects\exe-std\target\debug\exe-std.exe"#;
    // create_process(path, None, CREATE_NO_WINDOW | DETACHED_PROCESS).unwrap();

    // Sleeper::sleep(100000);
    // loop {
    //     Sleeper::sleep(1000);
    // }

    0
}

