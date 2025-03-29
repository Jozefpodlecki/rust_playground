
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Unknown,
    ProcessNotRunning,
    ProcessRunning,
    ProcessNotListening,
    ProcessListening(String),
    ProcesStopped
}

pub enum UpdateStatus {
    Unknown,
    NewVersion,
    LatestVersion,
    Error
}

pub enum Action {
    Task
}

pub struct AppState {
    pub region: Option<String>
}