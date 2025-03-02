use std::{ffi::{c_char, CStr}, sync::{mpsc::Receiver, Arc}};
use libloading::{Library, Symbol};
use shared::{Payload, User};

type TestMpscWithEnumFn = unsafe extern "C" fn() -> *mut Receiver<Payload>;
type GetUsersFn = unsafe extern "C" fn(*mut usize) -> *mut User;
type GetMessageFn = unsafe extern "C" fn() -> *mut c_char;

pub struct Wrapper {
    lib: Library,
    receiver: Option<*mut Receiver<Payload>>, 
}

impl Wrapper {
    pub fn new(dll_name: &str) -> Self {
        let lib = unsafe { Library::new(dll_name).unwrap() };

        Self {
            lib,
            receiver: None
        }
    }

    pub fn get_users(&self) -> Vec<User> {
        let get_users: Symbol<GetUsersFn> = unsafe { self.lib.get(b"get_users").unwrap() };

        let mut length: usize = 0;
        let users_ptr = unsafe { get_users(&mut length) };

        if users_ptr.is_null() || length == 0 {
            return vec![];
        }

        let users = unsafe { Vec::from_raw_parts(users_ptr, length, length) };

        users
    }

    pub fn get_message(&self) -> Option<String> {
        let get_message: Symbol<GetMessageFn> = unsafe { self.lib.get(b"get_message").unwrap() };

        unsafe {
            let msg_ptr = get_message();
            if msg_ptr.is_null() {
                return None;
            }

            let message = CStr::from_ptr(msg_ptr).to_string_lossy().into_owned();

            Some(message)
        }
    }

    pub fn set_test_mpsc_with_enum(&mut self) {
        let test_mpsc_with_enum: Symbol<TestMpscWithEnumFn> = unsafe { self.lib.get(b"test_mpsc_with_enum").unwrap() };

        self.receiver = unsafe { Some(test_mpsc_with_enum()) };
    }

    pub fn recv(&self) -> Option<Payload> {
        if let Some(rx_ptr) = self.receiver {

            if rx_ptr.is_null() {
                return None;
            }

            let rx = unsafe { &*rx_ptr };

            return rx.recv().ok()
        }

        None
    }
}

impl Drop for Wrapper {
    fn drop(&mut self) {
        if let Some(rx_ptr) = self.receiver.take() {
            if !rx_ptr.is_null() {
                unsafe { drop(Box::from_raw(rx_ptr)) }
            }
        }
    }
}
