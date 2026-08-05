#![no_std]
#![no_main]
#![windows_subsystem = "console"]
#![allow(unused)]

use core::{arch::naked_asm, panic::PanicInfo, ptr::null_mut};

use ntapi::ntrtl::{RtlCreateProcessParametersEx, RtlGetExePath, RtlGetNtSystemRoot};
use toolkit::{ProcessEnvironmentBlock, U16CStackString, println};

use crate::{flags::ProcessParametersFlags, string::UnicodeString};

extern crate builtins;

mod flags;
mod string;

#[inline(never)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
     println!("{info}");
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn mainCRTStartup() -> i32 {
    unsafe {
        let str = UnicodeString::from_str("abcdef").unwrap();

        println!("{str:?}");

        let system_root = unsafe { RtlGetNtSystemRoot() };
        let system_root = U16CStackString::<260>::from_ptr(system_root as _).unwrap();

        println!("system_root: {}", system_root);

        let peb = ProcessEnvironmentBlock::current_process();
        let mut current_params = *peb.process_params_raw();

        let cur_dir = U16CStackString::<260>::from_raw_parts(
            current_params.CurrentDirectory.DosPath.Buffer,
            current_params.CurrentDirectory.DosPath.Length as _);
        println!("cur_dir: {:?} handle: {:p}", cur_dir, current_params.CurrentDirectory.Handle);

        let window_title = U16CStackString::<260>::from_raw_parts(current_params.WindowTitle.Buffer, current_params.WindowTitle.Length as _).unwrap();
        println!("window_title: {}", window_title);

        let dll_path = U16CStackString::<260>::from_raw_parts(current_params.DllPath.Buffer, current_params.DllPath.Length as _);
        println!("dll_path: {:?}", dll_path);

        let desktop_info = U16CStackString::<260>::from_raw_parts(current_params.DesktopInfo.Buffer, current_params.DesktopInfo.Length as _).unwrap();
        println!("desktop_info: {}", desktop_info);

        let shell_info = U16CStackString::<260>::from_raw_parts(current_params.ShellInfo.Buffer, current_params.ShellInfo.Length as _).unwrap();
        println!("shell_info: {}", shell_info);

        let runtime_data = U16CStackString::<260>::from_raw_parts(current_params.RuntimeData.Buffer, current_params.RuntimeData.Length as _);
        println!("runtime_data: {:?}", runtime_data);

        let flags: ProcessParametersFlags = current_params.Flags.into();
        println!("{}", flags);

        let mut params: *mut ntapi::ntrtl::RTL_USER_PROCESS_PARAMETERS = null_mut();
        let status = RtlCreateProcessParametersEx(
            &mut params,
            &mut current_params.ImagePathName,
            // &mut current_params.DllPath,
            null_mut(),
            &mut current_params.CurrentDirectory.DosPath,
            &mut current_params.CommandLine,
            // current_params.Environment,
            null_mut(),
            &mut current_params.WindowTitle,
            &mut current_params.DesktopInfo,
            // &mut current_params.ShellInfo,
            null_mut(),
            // &mut current_params.RuntimeData,
            null_mut(),
            current_params.Flags);

        println!("RtlCreateProcessParametersEx 0x{:X}", status);
            
        //      ImagePathName: PUNICODE_STRING,
        // DllPath: PUNICODE_STRING,
        // CurrentDirectory: PUNICODE_STRING,
        // CommandLine: PUNICODE_STRING,
        // Environment: PVOID,
        // WindowTitle: PUNICODE_STRING,
        // DesktopInfo: PUNICODE_STRING,
        // ShellInfo: PUNICODE_STRING,
        // RuntimeData: PUNICODE_STRING,
        // Flags: ULONG,

        // let exe_path = unsafe { RtlGetExePath() };
        // let exe_path = U16CStackString::<260>::from_ptr(exe_path as _).unwrap();

        // println!("exe_path: {}", exe_path);
    }

    0
}