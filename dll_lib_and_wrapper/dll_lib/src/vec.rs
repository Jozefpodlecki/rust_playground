use std::mem;

use shared::User;

#[unsafe(no_mangle)]
pub extern "C" fn get_users(out_len: *mut usize) -> *mut User {
    let vec = vec![
        User { id: 1, name: "test".into() },
    ];

    unsafe {
        if !out_len.is_null() {
            *out_len = vec.len();
        }
    }

    let ptr = vec.as_ptr();
    mem::forget(vec);
    ptr as *mut User
}