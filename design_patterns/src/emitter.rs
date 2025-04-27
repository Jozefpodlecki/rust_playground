use anyhow::Result;
use serde::Serialize;

#[derive(Debug, Serialize)]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "lowercase")]
pub enum Event {
    Start,
    Stop,
    Custom(String),
}

pub trait Emitter {
    fn emit(&self, event: Event) -> Result<()>;
}

pub struct DefaultEmitter;

impl DefaultEmitter {
    pub fn new() -> Self {
        DefaultEmitter
    }
}

impl Emitter for DefaultEmitter {
    fn emit(&self, event: Event) -> Result<()> {
        let json = serde_json::to_string(&event)?;
        println!("{}", json);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit() {
        let emitter = DefaultEmitter::new();
        let emitter_obj: &dyn Emitter = &emitter;
        // emitter.emit(Event::Start).unwrap();
        // emitter.emit(Event::Custom("Hello World".to_string())).unwrap();

        let (data, vtable): (*const (), *const ()) = unsafe {
            std::mem::transmute(emitter_obj)
        };
    
        println!("Data pointer: {:p}, Vtable pointer: {:p}", data, vtable);
    }
}
