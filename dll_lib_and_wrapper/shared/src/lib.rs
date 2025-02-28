use rand::Rng;

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