use std::arch::asm;

use core::ffi::c_void;

pub type PVOID = *mut c_void;
pub type ULONG = u32;
pub type BYTE = u8;
pub type USHORT = u16;

#[repr(C)]
pub struct PEB {
    pub Reserved1: [BYTE; 2],
    pub BeingDebugged: BYTE,
    pub Reserved2: BYTE,
    pub Reserved3: [PVOID; 2],
    pub Ldr: *mut PEB_LDR_DATA,
    pub ProcessParameters: *mut RTL_USER_PROCESS_PARAMETERS,
    pub Reserved4: [PVOID; 3],
    pub AtlThunkSListPtr: PVOID,
    pub Reserved5: PVOID,
    pub Reserved6: ULONG,
    pub Reserved7: PVOID,
    pub Reserved8: ULONG,
    pub AtlThunkSListPtr32: ULONG,
    pub Reserved9: [PVOID; 45],
    pub Reserved10: [BYTE; 96],
    pub PostProcessInitRoutine: PVOID,
    pub Reserved11: [BYTE; 128],
    pub Reserved12: [PVOID; 1],
    pub SessionId: ULONG,
}

#[repr(C)]
pub struct LIST_ENTRY {
    pub Flink: *mut LIST_ENTRY,
    pub Blink: *mut LIST_ENTRY,
}


#[repr(C)]
pub struct UNICODE_STRING {
    pub Length: u16,
    pub MaximumLength: u16,
    pub Buffer: *mut u16,
}

#[repr(C)]
pub struct RTL_USER_PROCESS_PARAMETERS {
    pub Reserved1: [BYTE; 16],
    pub Reserved2: [PVOID; 10],
    pub ImagePathName: UNICODE_STRING,
    pub CommandLine: UNICODE_STRING,
}

#[repr(C)]
pub struct LDR_DATA_TABLE_ENTRY {
    pub InLoadOrderLinks: LIST_ENTRY,
    pub InMemoryOrderLinks: LIST_ENTRY,
    pub InInitializationOrderLinks: LIST_ENTRY,
    pub DllBase: PVOID,
    pub EntryPoint: PVOID,
    pub SizeOfImage: ULONG,
    pub FullDllName: UNICODE_STRING,
    pub BaseDllName: UNICODE_STRING,
    pub Flags: ULONG,
    pub LoadCount: USHORT,
    pub TlsIndex: USHORT,
    pub HashLinks: LIST_ENTRY,
    pub TimeDateStamp: ULONG,
    // (rest omitted — not needed for enumeration)
}

#[repr(C)]
pub struct PEB_LDR_DATA {
    pub Length: ULONG,
    pub Initialized: BYTE,
    pub Reserved1: [BYTE; 3],
    pub SsHandle: PVOID,

    pub InLoadOrderModuleList: LIST_ENTRY,
    pub InMemoryOrderModuleList: LIST_ENTRY,
    pub InInitializationOrderModuleList: LIST_ENTRY,
}

fn get_peb() -> *mut PEB {
    let peb: *mut PEB;
    unsafe {
        asm!(
            "mov {0}, gs:[0x60]",
            out(reg) peb,
        );
    }
    peb
}

unsafe fn read_unicode_string(us: &UNICODE_STRING) -> String {
    if us.Buffer.is_null() || us.Length == 0 {
        return String::new();
    }
    let slice = std::slice::from_raw_parts(us.Buffer, (us.Length / 2) as usize);
    String::from_utf16_lossy(slice)
}

fn main() {
    let peb = get_peb();

    unsafe {
        let ldr = (*peb).Ldr;
        let list_head = &(*ldr).InMemoryOrderModuleList as *const LIST_ENTRY as *mut LIST_ENTRY;
        let mut current = (*list_head).Flink;

        while current != list_head {
            let entry = (current as *const u8)
                .sub( std::mem::size_of::<LIST_ENTRY>())
                as *const LDR_DATA_TABLE_ENTRY;

            let entry_ref = &*entry;

            let dll_name = read_unicode_string(&entry_ref.BaseDllName);

            println!(
                "Module: {:p}  Name: {}  Size: {} bytes",
                entry_ref.DllBase, dll_name, entry_ref.SizeOfImage
            );

            current = (*current).Flink;
        }
    }
}
