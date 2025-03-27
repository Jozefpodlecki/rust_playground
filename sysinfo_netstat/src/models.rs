
#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Unknown,
    ProcessNotRunning,
    ProcessRunning,
    ProcessListening(String),
    ProcesStopped
}