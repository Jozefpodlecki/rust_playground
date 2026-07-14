use core::ptr;

use alloc::boxed::Box;
use toolkit::{U16CStackString, println};
use winapi::{shared::{minwindef::{BOOL, HINSTANCE, LPARAM, LPVOID, LRESULT, PUINT, UINT, WPARAM}, windef::HWND}, um::{libloaderapi::GetModuleHandleW, winuser::{CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GetMessageW, GetRawInputData, HRAWINPUT, HWND_MESSAGE, KEYBOARD_OVERRUN_MAKE_CODE, MSG, PCRAWINPUTDEVICE, PostQuitMessage, RAWINPUT, RAWINPUTHEADER, RID_INPUT, RIDEV_INPUTSINK, RIDEV_NOLEGACY, RIM_TYPEKEYBOARD, RIM_TYPEMOUSE, RegisterClassW, RegisterRawInputDevices, WM_DESTROY, WM_INPUT, WNDCLASSW, WS_OVERLAPPEDWINDOW}}};

pub const HID_USAGE_PAGE_GENERIC: u16 = 0x01;
pub const HID_USAGE_GENERIC_MOUSE: u16 = 0x02;
pub const HID_USAGE_GENERIC_KEYBOARD: u16 = 0x06;

#[link(name = "win32u", kind = "raw-dylib")]
unsafe extern "system" {
    pub fn NtUserRegisterRawInputDevices(
        pRawInputDevices: PCRAWINPUTDEVICE,
        uiNumDevices: UINT,
        cbSize: UINT,
    ) -> BOOL;
    pub fn NtUserGetRawInputData(
        hRawInput: HRAWINPUT,
        uiCommand: UINT,
        pData: LPVOID,
        pcbSize: PUINT,
        cbSizeHeader: UINT,
    ) -> UINT;
}

extern "system" fn wnd_proc(
    hwnd: HWND,
    msg: UINT,
    w_param: WPARAM,
    l_param: LPARAM,
) -> LRESULT {
    match msg {
        WM_INPUT => {
            // let result = handle_raw_input(l_param);
            return 0;
        }
        WM_DESTROY => {
            unsafe { PostQuitMessage(0); }
            return 0;
        }
        _ => unsafe { return DefWindowProcW(hwnd, msg, w_param, l_param); },
    }
}

fn handle_raw_input(l_param: LPARAM) -> LRESULT {
    unsafe {

        let mut size: UINT = 0;
        GetRawInputData(
            l_param as _,
            RID_INPUT,
            ptr::null_mut(),
            &mut size,
            core::mem::size_of::<RAWINPUTHEADER>() as _,
        );

        if size == 0 {
            return 0;
        }
        
        // cannot be stack allocated!
        let mut buffer: Box<[u8]> = Box::<[u8]>::new_zeroed_slice(size as usize).assume_init();

        if size as usize > buffer.len() {
            return 0;
        }

        if NtUserGetRawInputData(
            l_param as _,
            RID_INPUT,
            buffer.as_mut_ptr() as _,
            &mut size,
            core::mem::size_of::<RAWINPUTHEADER>() as _,
        ) == !0u32 {
            return 0;
        }

        let raw = &*(buffer.as_ptr() as *const RAWINPUT);
        process_raw_input(raw);
        0
    }
}

fn process_raw_input(raw: &RAWINPUT) {
    match raw.header.dwType {
        RIM_TYPEKEYBOARD => {
            unsafe { process_keyboard(&*raw.data.keyboard()); }
        }
        RIM_TYPEMOUSE => {
            unsafe { process_mouse(&*raw.data.mouse()); }
        }
        _ => {}
    }
}

fn process_keyboard(kbd: &winapi::um::winuser::RAWKEYBOARD) {
    unsafe {
        let vk_code = kbd.VKey;
        let scan_code = kbd.MakeCode;
        let flags = kbd.Flags;
        let is_down = (flags & 0x01) == 0;
        let is_up = (flags & 0x01) != 0;
        let is_e0 = (flags & 0x02) != 0;
        let is_e1 = (flags & 0x04) != 0;

        if vk_code == KEYBOARD_OVERRUN_MAKE_CODE as _ {
            return;
        }

        let ch = match vk_code {
            0x30..=0x39 => (b'0' + (vk_code - 0x30) as u8) as char,
            0x41..=0x5A => (b'A' + (vk_code - 0x41) as u8) as char,
            _ => '?',
        };

        println!(
            "Key: VK={} (0x{:02X}) scan={} down={} up={} e0={} e1={} char={}",
            vk_code, vk_code, scan_code, is_down, is_up, is_e0, is_e1, ch
        );
    }
}

fn process_mouse(mouse: &winapi::um::winuser::RAWMOUSE) {
    unsafe {
        println!(
            "Mouse: dx={} dy={} buttons={:#x}",
            mouse.lLastX, mouse.lLastY, mouse.usButtonFlags
        );
    }
}

fn create_hidden_window(
    class_name: &U16CStackString<20>,
    hinstance: HINSTANCE,
) -> HWND {
    unsafe {
        let wc = WNDCLASSW {
            style: 0,
            lpfnWndProc: Some(wnd_proc),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: hinstance,
            hIcon: ptr::null_mut(),
            hCursor: ptr::null_mut(),
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null(),
            lpszClassName: class_name.as_ptr(),
        };

        if RegisterClassW(&wc) == 0 {
            return ptr::null_mut();
        }

        let hwnd = CreateWindowExW(
            0,
            class_name.as_ptr(),
            class_name.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            0,
            0,
            0,
            0,
            HWND_MESSAGE,
            ptr::null_mut(),
            hinstance,
            ptr::null_mut(),
        );

        hwnd
    }
}

fn register_raw_input_devices(hwnd: HWND) -> bool {
    unsafe {
        use winapi::um::winuser::RAWINPUTDEVICE;

        let devices = [
             RAWINPUTDEVICE {
                usUsagePage: HID_USAGE_PAGE_GENERIC,
                usUsage: HID_USAGE_GENERIC_MOUSE,
                dwFlags: RIDEV_NOLEGACY | RIDEV_INPUTSINK,
                hwndTarget: hwnd
            },
            RAWINPUTDEVICE {
                usUsagePage: HID_USAGE_PAGE_GENERIC,
                usUsage: HID_USAGE_GENERIC_KEYBOARD,
                dwFlags: RIDEV_NOLEGACY | RIDEV_INPUTSINK,
                hwndTarget: hwnd,
            },
        ];

        NtUserRegisterRawInputDevices(
            devices.as_ptr(),
            devices.len() as u32,
            core::mem::size_of::<RAWINPUTDEVICE>() as u32,
        ) != 0
    }
}

fn run_message_loop() {
    unsafe {
        let mut msg: MSG = core::mem::zeroed();
        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) != 0 {
            if msg.message == WM_INPUT {
                let result = handle_raw_input(msg.lParam);
            }
            DispatchMessageW(&msg);
        }
    }
}

pub fn run_scenario_raw_input_device() -> i32 {
    unsafe {
        let hinstance = GetModuleHandleW(ptr::null());
        let class_name = U16CStackString::<20>::from_str("rawinput").unwrap();

        let hwnd = create_hidden_window(&class_name, hinstance);
        if hwnd.is_null() {
            return 1;
        }

        if !register_raw_input_devices(hwnd) {
            DestroyWindow(hwnd);
            return 1;
        }

        run_message_loop();
        DestroyWindow(hwnd);
    }

    0
}