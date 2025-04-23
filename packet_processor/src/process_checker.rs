use tokio::sync::watch;

#[derive(Default, Clone)]
pub enum ProcessStatus {
    #[default]
    Idle,
    NotFound,
    Stopped,
    Running,
    Listening
}

pub struct ProcessChecker {
    current: ProcessStatus,
    tx: watch::Sender<ProcessStatus>,
}

impl ProcessChecker {
    pub fn new() -> Self {
        let current = ProcessStatus::Idle;
        let (tx, _) = watch::channel(current.clone());
        Self {
            current,
            tx
        }
    }

    pub fn update(&mut self, new_status: ProcessStatus) -> anyhow::Result<()> {
        self.tx.send(new_status)?;
        Ok(())
    }

    pub fn subscribe(&self) -> watch::Receiver<ProcessStatus> {
        self.tx.subscribe()
    }
}