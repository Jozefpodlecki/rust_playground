use std::ffi::{CString, CStr};
use std::os::raw::c_char;

#[unsafe(no_mangle)]
pub extern "C" fn get_message() -> *mut c_char {
    let message = CString::new("Random cstr").expect("CString::new failed");
    message.into_raw()
}
