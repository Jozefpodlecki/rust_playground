use core::fmt;

#[derive(Debug)]
pub enum ProcessBuilderError {
    BufferOverflow,
    CouldNotOpenFile,
    InvalidImageFormat,
    MachineMismatch,
    ImageFileExecutionOptions,
    CouldNotCreateSection,
    ImagePathEmpty,
    NtOpenProcessFailed(i32),
    InvalidParameter,
    NtCreateUserProcessFailed(i32),
    AttributesBuildFailed,
    ProcessParamsBuildFailed,
    EnvironmentBuildFailed,
}

impl ProcessBuilderError {
    pub fn from_ntstatus(status: i32) -> Self {
        ProcessBuilderError::NtCreateUserProcessFailed(status)
    }
}

impl From<i32> for ProcessBuilderError {
    fn from(status: i32) -> Self {
        ProcessBuilderError::NtCreateUserProcessFailed(status)
    }
}

impl fmt::Display for ProcessBuilderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessBuilderError::BufferOverflow => write!(f, "Buffer overflow: insufficient buffer size"),
            ProcessBuilderError::ImagePathEmpty => write!(f, "Image path is empty"),
            ProcessBuilderError::NtOpenProcessFailed(status) => {
                write!(f, "NtOpenProcess failed with status: 0x{:08X}", status)
            }
            ProcessBuilderError::InvalidParameter => write!(f, "Invalid parameter provided"),
            ProcessBuilderError::NtCreateUserProcessFailed(status) => {
                write!(f, "NtCreateUserProcess failed with status: 0x{:08X}", status)
            }
            ProcessBuilderError::AttributesBuildFailed => write!(f, "Failed to build process attributes"),
            ProcessBuilderError::ProcessParamsBuildFailed => write!(f, "Failed to build process parameters"),
            ProcessBuilderError::EnvironmentBuildFailed => write!(f, "Failed to build environment block"),
            ProcessBuilderError::CouldNotOpenFile => write!(f, "Failed to open file"),
            ProcessBuilderError::InvalidImageFormat => write!(f, "Invalid image format"),
            ProcessBuilderError::CouldNotCreateSection => write!(f, "Could not create section"),
            ProcessBuilderError::MachineMismatch => write!(f, "Machine mismatch"),
            ProcessBuilderError::ImageFileExecutionOptions => write!(f, "ImageFileExecutionOptions?"),
        }
    }
}

impl core::error::Error for ProcessBuilderError {}