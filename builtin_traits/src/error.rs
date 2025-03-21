use std::{error::Error, fmt};

#[derive(Debug)]
pub struct CustomError {
    pub message: String,
}

impl fmt::Display for CustomError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "CustomError: {}", self.message)
    }
}

impl Error for CustomError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }
}

// impl FromResidual<Box<dyn Error>> for CustomError {
//     fn from_residual(residual: Box<dyn Error>) -> Self {
//         CustomError {
//             message: format!("Wrapped error: {}", residual),
//         }
//     }
// }


impl CustomError {
    pub fn new(message: &str) -> Self {
        CustomError {
            message: message.to_string(),
        }
    }
}

fn throw_error_case_1() -> anyhow::Result<()> {
    Err(CustomError::new("Something went wrong").into())
}

fn throw_error_case_2() -> Result<(), CustomError> {
    Err(CustomError::new("Something went wrong"))
}

fn throw_error_case_3() -> Result<(), Box<dyn Error>> {
    Err(CustomError::new("Something went wrong").into())
}