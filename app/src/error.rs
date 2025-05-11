use log::error;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Generic error: {0}")]
    Generic(#[from] Box<dyn std::error::Error>),
    #[error("Serialization: {0}")]
    Serde(#[from] serde_json::error::Error),
    #[error("Command: {0}")]
    Command(String),
    #[error("Sqlite: {0}")]
    Sqlite(String),
    #[error("Unknown error")]
    Unknown,
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        error!("{}", self);
        serializer.serialize_str(self.to_string().as_ref())
    }
}
