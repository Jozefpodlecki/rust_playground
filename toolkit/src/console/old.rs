




pub fn get_console_encoding(handle: HANDLE) -> Option<u32> {
    unsafe {
        let file_type = GetFileType(handle);
        if file_type != FILE_TYPE_CHAR {
            return None;
        }

        let mut mode: DWORD = 0;
        let is_console = GetConsoleMode(handle, &mut mode) != 0;

        if is_console {
            // Input or output?
            // GetConsoleMode succeeds for both input and output
            // Check if it's an input console by trying GetNumberOfConsoleInputEvents
            // Or simply assume output for stdout
            Some(GetConsoleOutputCP())
        } else {
            // Not a console handle
            None
        }
    }
}

pub fn get_output_encoding(handle: HANDLE) -> u32 {
    unsafe {
        let file_type = GetFileType(handle);
        if file_type == FILE_TYPE_CHAR {
            let mut mode: DWORD = 0;
            if GetConsoleMode(handle, &mut mode) != 0 {
                return GetConsoleOutputCP();
            }
        }
        // Fallback - default to UTF-8
        CP_UTF8
    }
}

pub fn is_console_utf8(handle: HANDLE) -> bool {
    get_output_encoding(handle) == CP_UTF8
}

