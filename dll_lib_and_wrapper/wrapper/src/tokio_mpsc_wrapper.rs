use tokio::sync::mpsc::{self, UnboundedReceiver};
use libloading::{Library, Symbol};
use shared::Payload;

type TokioMpscFn = unsafe extern "C" fn() -> *mut UnboundedReceiver<Payload>;

pub struct TokioMpscWrapper<'a> {
    dll_name: &'a str,
    lib: Option<Library>,
    receiver: Option<*mut UnboundedReceiver<Payload>>, 
}

impl<'a> TokioMpscWrapper<'a> {
    pub fn new(dll_name: &'a str) -> Self {
        
        Self {
            dll_name,
            lib: None,
            receiver: None
        }
    }

    pub fn load(&mut self) {
        let lib = unsafe { Library::new(self.dll_name).unwrap() };
        let mpsc: Symbol<TokioMpscFn> = unsafe { lib.get(b"tokio_mpsc").unwrap() };
        self.receiver = unsafe { Some(mpsc()) };
            
        self.lib = Some(lib);
    }

    pub async fn recv(&self) -> Option<Payload> {
        if let Some(rx_ptr) = self.receiver {

            if rx_ptr.is_null() {
                return None;
            }

            let mut rx = unsafe { &mut * rx_ptr };

            return rx.recv().await;
        }

        None
    }
}

impl<'a> Drop for TokioMpscWrapper<'a> {
    fn drop(&mut self) {
        if let Some(rx_ptr) = self.receiver.take() {
            if !rx_ptr.is_null() {
                unsafe { drop(Box::from_raw(rx_ptr)) }
            }
        }

        if let Some(lib) = self.lib.take() {
            lib.close();
        }
    }
}
