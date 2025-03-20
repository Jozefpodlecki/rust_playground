
use std::pin::Pin;

use shared::{BackgroundService, BackgroundServiceWrapper};
use anyhow::*;

pub struct DefaultBackgroundService {

}

impl BackgroundService for DefaultBackgroundService {
    fn start(&mut self) -> Result<()> {
        println!("start");
        Ok(())
    }

    fn stop(&mut self) -> Result<()> {
        println!("stop");
        Ok(())
    }
}

impl DefaultBackgroundService {
    pub fn new() -> Self {
        Self {}
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn get_service() -> Box<dyn BackgroundService> {
    Box::new(DefaultBackgroundService::new())
}

#[unsafe(no_mangle)]
pub extern "C" fn get_service_raw() -> *mut dyn BackgroundService {
    Box::into_raw(Box::new(DefaultBackgroundService::new()))
}

#[unsafe(no_mangle)]
pub extern "C" fn get_service_raw_pin() -> Pin<Box<dyn BackgroundService>> {
    Box::pin(DefaultBackgroundService::new())
}

#[unsafe(no_mangle)]
pub extern "C" fn get_service_wrapped() -> *mut BackgroundServiceWrapper {
    let wrapper = BackgroundServiceWrapper {
        version: 1,
        service: Box::new(DefaultBackgroundService::new())
    };

    Box::into_raw(Box::new(wrapper))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_return_service() {
        let service = get_service_raw();
        let service = unsafe { &mut *service }; 
        
        service.start().unwrap();
        service.stop().unwrap();
    }
}