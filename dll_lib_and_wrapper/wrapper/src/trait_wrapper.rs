use std::pin::Pin;

use tokio::sync::mpsc::{UnboundedReceiver};
use libloading::{Library, Symbol};
use shared::{BackgroundService, BackgroundServiceWrapper, Payload};

type GetServiceFn = unsafe extern "C" fn() -> Box<dyn BackgroundService>;
type GetServiceRawFn = unsafe extern "C" fn() -> *mut dyn BackgroundService;
type GetServiceRawPinFn = unsafe extern "C" fn() -> Pin<Box<dyn BackgroundService>>;
type GetServiceWrappedFn = unsafe extern "C" fn() -> *mut BackgroundServiceWrapper;

pub struct TraitWrapper<'a> {
    dll_name: &'a str,
}

impl<'a> TraitWrapper<'a> {
    pub fn new(dll_name: &'a str) -> Self {
        
        Self {
            dll_name,
        }
    }

    pub fn get_service(&mut self) -> Box<dyn BackgroundService> {
        let lib = unsafe { Library::new(self.dll_name).unwrap() };
        let get_service: Symbol<GetServiceFn> = unsafe { lib.get(b"get_service").unwrap() };
        let service = unsafe { Some(get_service()) };
            
        service.unwrap()
    }

    pub fn get_service_raw(&mut self) -> *mut dyn BackgroundService {
        let lib = unsafe { Library::new(self.dll_name).unwrap() };
        let get_service: Symbol<GetServiceRawFn> = unsafe { lib.get(b"get_service_raw").unwrap() };
        let service = unsafe { get_service() };
        println!("before_unwrap");
        service
    }

    pub fn get_service_raw_pin(&mut self) -> Pin<Box<dyn BackgroundService>> {
        let lib = unsafe { Library::new(self.dll_name).unwrap() };
        let get_service: Symbol<GetServiceRawPinFn> = unsafe { lib.get(b"get_service_raw_pin").unwrap() };
        let service = unsafe { get_service() };
        println!("before_unwrap");
        service
    }

    pub fn get_service_wrapped(&mut self) -> *mut BackgroundServiceWrapper {
        let lib = unsafe { Library::new(self.dll_name).unwrap() };
        let get_service: Symbol<GetServiceWrappedFn> = unsafe { lib.get(b"get_service_wrapped").unwrap() };
        let service = unsafe { get_service() };
        println!("before_unwrap");
        service
    }
}

impl<'a> Drop for TraitWrapper<'a> {
    fn drop(&mut self) {
       
    }
}
