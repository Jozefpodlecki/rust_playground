use rand::Rng;
use anyhow::*;

pub trait BackgroundService {
    fn start(&mut self) -> Result<()>;
    fn stop(&mut self) -> Result<()>;
}

pub struct BackgroundServiceWrapper {
    pub version: i64,
    pub service: Box<dyn BackgroundService>
}

#[repr(C)]
#[derive(Debug)]
pub struct User {
    pub id: i32,
    pub name: String,
}


#[derive(Debug)]
pub enum TaskState {
    InProgress
}

#[repr(C)]
#[derive(Debug)]
pub enum Payload {
    None,
    NewTask {
        id: u64,
        name: String
    },
    Update {
        id: u64,
        state: TaskState
    },
    Completed {
        id: u64,
    }
}

impl Payload {
    pub fn random() -> Self {
        let mut rng = rand::rng();
        let choice = rng.random_range(0..4);

        match choice {
            0 => Payload::None,
            1 => Payload::NewTask {
                id: rng.random(),
                name: format!("Task-{}", rng.random_range(1..100)),
            },
            2 => Payload::Update {
                id: rng.random(),
                state: TaskState::InProgress,
            },
            3 => Payload::Completed {
                id: rng.random(),
            },
            _ => unreachable!(),
        }
    }
}