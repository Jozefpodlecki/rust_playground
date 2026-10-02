use logger::LoggerError;
use win_platform::NtError;

#[derive(Debug)]
pub enum IpcError {
    Logger(LoggerError),
    CouldNotCreate(NtError),
    CouldNotListen(NtError),
    CouldNotConnect(NtError),
    CouldNotRead(NtError),
    CouldNotWrite(NtError),
    Disconnected,
    Deserialize(FrameError),
    Serialize(FrameError),
}

impl From<NtError> for IpcError {
    fn from(value: NtError) -> Self {
        IpcError::CouldNotCreate(value)
    }
}

impl core::fmt::Display for IpcError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            IpcError::Logger(e) => write!(f, "{e}"),
            IpcError::CouldNotCreate(e) => write!(f, "could not create pipe: {e}"),
            IpcError::CouldNotListen(e) => write!(f, "could not listen to pipe: {e}"),
            IpcError::CouldNotConnect(e) => write!(f, "could not connect to pipe: {e}"),
            IpcError::CouldNotRead(e) => write!(f, "could not read from pipe: {e}"),
            IpcError::CouldNotWrite(e) => write!(f, "could not write to pipe: {e}"),
            IpcError::Disconnected => write!(f, "pipe disconnected"),
            IpcError::Deserialize(e) => write!(f, "could not deserialize frame: {e}"),
            IpcError::Serialize(e) => write!(f, "could not serialize frame: {e}"),
        }
    }
}

impl core::error::Error for IpcError {}


#[derive(Debug)]
pub enum FrameError {
    TooLong { got: usize, max: usize },
    Serialize(postcard::Error),
    Deserialize(postcard::Error),
}

impl core::fmt::Display for FrameError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            FrameError::TooLong { got, max } => {
                write!(f, "frame too long: {got} bytes, max {max}")
            }
            FrameError::Serialize(e) => write!(f, "serialize error: {e}"),
            FrameError::Deserialize(e) => write!(f, "deserialize error: {e}"),
        }
    }
}

impl core::error::Error for FrameError {}