use core::ptr;

use ntapi::ntpsapi::NtCurrentThreadId;
use toolkit::{Sleeper, println};
use winapi::{ctypes::c_int, shared::{minwindef::{BOOL, DWORD, HINSTANCE, LPARAM, LRESULT, TRUE, UINT, WPARAM}, ntdef::{INT, UNICODE_STRING}, windef::{HHOOK, HWND}}, um::winuser::{CallNextHookEx, GetForegroundWindow, GetMessageA, GetWindowThreadProcessId, HOOKPROC, KBDLLHOOKSTRUCT, MSG, MSLLHOOKSTRUCT, PMSG, UnhookWindowsHookEx, WH_KEYBOARD_LL, WH_MOUSE_LL}};

#[link(name = "win32u", kind = "raw-dylib")]
unsafe extern "system" {
    pub fn NtUserSetWindowsHookEx(
        Mod: HINSTANCE,
        UnsafeModuleName: *mut UNICODE_STRING,
        ThreadId: DWORD,
        HookId: c_int,
        HookProc: HOOKPROC,
        Ansi: BOOL,
    ) -> HHOOK;
    pub fn NtUserGetMessage(
        pMsg: PMSG,
        hWnd: HWND,
        MsgFilterMin: UINT,
        MsgFilterMax: UINT) -> BOOL;
    pub fn NtUserUnhookWindowsHookEx(hook: HHOOK) -> BOOL;
    pub fn NtUserCallNextHookEx(
        hhk: HHOOK,
        nCode: c_int,
        wParam: WPARAM,
        lParam: LPARAM,
    ) -> LRESULT;
}

static mut HOOK: Option<winapi::shared::windef::HHOOK> = None;

pub fn run_scenario_set_windows_hook() -> i32 {
    unsafe {
        let hook = NtUserSetWindowsHookEx(
            ptr::null_mut(),      // Mod (HINSTANCE) - NULL for WH_KEYBOARD_LL
            ptr::null_mut(),
            0,
            // WH_KEYBOARD_LL,
            WH_MOUSE_LL,
            Some(mouse_callback),
            1,
        );
        
        if hook.is_null() {
            return 1;
        }
        
        HOOK = Some(hook);
    }
    
    unsafe {
        let mut message: MSG = core::mem::zeroed();
        
        let result = NtUserGetMessage(
            &mut message,
            ptr::null_mut(),
            0,
            0,
        );
        

    }
    unsafe {
        if let Some(hook) = HOOK {
            NtUserUnhookWindowsHookEx(hook);
        }
    }

    0
}

unsafe extern "system" fn mouse_callback(
    code: c_int,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let mouse_struct = &*(l_param as *const MSLLHOOKSTRUCT);

        let foreground_hwnd = GetForegroundWindow();
        let mut target_pid: DWORD = 0;
        let target_thread_id = GetWindowThreadProcessId(foreground_hwnd, &mut target_pid);

        match w_param as u32 {
            0x0200 => println!("WM_MOUSEMOVE: x={}, y={}", mouse_struct.pt.x, mouse_struct.pt.y),
            0x0201 => println!("WM_LBUTTONDOWN: x={}, y={}", mouse_struct.pt.x, mouse_struct.pt.y),
            0x0202 => println!("WM_LBUTTONUP: x={}, y={}", mouse_struct.pt.x, mouse_struct.pt.y),
            0x0204 => println!("WM_RBUTTONDOWN: x={}, y={}", mouse_struct.pt.x, mouse_struct.pt.y),
            0x0205 => println!("WM_RBUTTONUP: x={}, y={}", mouse_struct.pt.x, mouse_struct.pt.y),
            0x0207 => println!("WM_MBUTTONDOWN: x={}, y={}", mouse_struct.pt.x, mouse_struct.pt.y),
            0x0208 => println!("WM_MBUTTONUP: x={}, y={}", mouse_struct.pt.x, mouse_struct.pt.y),
            0x020A => println!("WM_MOUSEWHEEL: delta={}", mouse_struct.mouseData >> 16),
            _ => {}
        }
    }

    NtUserCallNextHookEx(ptr::null_mut(), code, w_param, l_param)
}

unsafe extern "system" fn keyboard_callback(
    code: c_int,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    if code >= 0 {
        let tid = unsafe { NtCurrentThreadId() as usize };
        let kbd_struct = &*(l_param as *const KBDLLHOOKSTRUCT);
        
        let foreground_hwnd = GetForegroundWindow();
        let mut target_pid: DWORD = 0;
        let target_thread_id = GetWindowThreadProcessId(foreground_hwnd, &mut target_pid);

        println!(
                "targetPid={}, targetThreadId={}, scanCode={}, vkCode={}",
                target_pid,
                target_thread_id,
                kbd_struct.scanCode,
                kbd_struct.vkCode
            );
        
    }
    
    unsafe { NtUserCallNextHookEx(ptr::null_mut(), code, w_param, l_param) }
}
