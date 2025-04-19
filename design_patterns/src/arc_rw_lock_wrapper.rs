use std::{sync::{Arc, RwLock}, thread};


pub struct Wrapper {
    inner: Arc<RwLock<Inner>>
}

impl Wrapper {
    pub fn new() -> Self {
        Self { inner: Arc::new(RwLock::new(Inner {  })) }
    }
}

pub struct Inner {}

impl Inner {
    pub fn run(&mut self) {
        
    }
}

impl Clone for Wrapper {
    fn clone(&self) -> Self {
        Wrapper {
            inner: Arc::clone(&self.inner),
        }
    }
}

pub fn test_clone() {
    let wrapper = Wrapper::new();

    {
        let wrapper = wrapper.clone();
        thread::spawn(move || {
            wrapper
        });
    }

    {
        thread::spawn(move || {
            wrapper.clone()
        });
    }
}