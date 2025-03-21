use std::ffi::CString;
use std::mem;
use std::ptr;

struct Unmanaged {
    message_ptr: *mut i8, 
}

impl Unmanaged {
    fn new(message: &str) -> Self {
        let c_string = CString::new(message).expect("CString::new failed");
        let message_ptr = c_string.into_raw(); 

        Unmanaged {
            message_ptr
        }
    }
}

impl Drop for Unmanaged {
    fn drop(&mut self) {
        unsafe {
            if !self.message_ptr.is_null() {
                let _ = CString::from_raw(self.message_ptr);
            }
        }
    }
}