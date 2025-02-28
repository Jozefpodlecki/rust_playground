use std::sync::{mpsc::Receiver, Arc};
use libloading::{Library, Symbol};
use shared::Payload;

type TestMpscWithEnumFn = unsafe extern "C" fn() -> *mut Receiver<Payload>;

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
