
pub fn enable_debug_privilege() -> bool {
    unsafe {
        let mut token_handle: HANDLE = core::ptr::null_mut();
        
        if NtOpenProcessToken(
            NtCurrentProcess,
            winapi::um::winnt::TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY,
            &mut token_handle
        ) != 0 {
            return false;
        }

        let mut luid = core::mem::zeroed();
        let privilege_name = U16CStackString::<50>::from_str("SeDebugPrivilege\0").unwrap();
        
        if LookupPrivilegeValueW(
            core::ptr::null(),
            privilege_name.as_ptr(),
            &mut luid
        ) == 0 {
            println!("LookupPrivilegeValueW");
            return false;
        }

        let mut tp = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: SE_PRIVILEGE_ENABLED,
            }],
        };

        let result = AdjustTokenPrivileges(
            token_handle,
            0,
            &mut tp,
            core::mem::size_of::<TOKEN_PRIVILEGES>() as u32,
            core::ptr::null_mut(),
            core::ptr::null_mut(),
        );

        winapi::um::handleapi::CloseHandle(token_handle);

        result != 0
    }
}
