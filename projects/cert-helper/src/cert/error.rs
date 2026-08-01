use core::fmt;

use crate::cert::types::*;

#[derive(Debug)]
pub enum UnsupportedKeyTypeReason {
    NotCng,
    UnsupportedAlgorithm,
}

#[derive(Debug)]
pub enum SigningFailedReason {
    InvalidParameter,
    NotSupported,
    InvalidKey,
    InternalError,
    Unknown(i32),
}

impl SigningFailedReason {
    pub fn from(value: i32) -> Self {
        let code = value as u32;
        println!("0x{:08X}", code);
        
        match code {
            0x80090027 => SigningFailedReason::InvalidParameter,  // NTE_INVALID_PARAMETER
            0x80090029 => SigningFailedReason::NotSupported,      // NTE_NOT_SUPPORTED
            0x80090003 => SigningFailedReason::InvalidKey,        // NTE_INVALID_KEY
            0x8009001A => SigningFailedReason::InternalError,     // NTE_INTERNAL_ERROR
            _ => SigningFailedReason::Unknown(value),
        }
    }
}


impl fmt::Display for SigningFailedReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SigningFailedReason::InvalidParameter => write!(f, "Invalid parameter - check padding info structure"),
            SigningFailedReason::NotSupported => write!(f, "Operation not supported - key may not support SHA256 with PKCS1 padding"),
            SigningFailedReason::InvalidKey => write!(f, "Invalid key handle"),
            SigningFailedReason::InternalError => write!(f, "Internal error in cryptographic provider"),
            SigningFailedReason::Unknown(code) => write!(f, "Unknown error: 0x{:08X}", code),
        }
    }
}

#[derive(Debug)]
pub enum CertStoreError {
    UnsupportedKeyType(UnsupportedKeyTypeReason),
    StoreOpenFailed { store_name: String, error_code: u32 },
    StoreCloseFailed { error_code: u32 },
    ThumbprintNotFound(Thumbprint, u32),
    CertificateNotFound { thumbprint: String, error_code: u32 },
    InvalidThumbprint { thumbprint: String, reason: String },
    InvalidStoreHandle,
    CertificateDataMissing,
    PrivateKeyAcquisitionFailed { error_code: u32 },
    SigningFailed(SigningFailedReason),
}

impl std::error::Error for CertStoreError {}

impl fmt::Display for CertStoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CertStoreError::UnsupportedKeyType(reason) => {
                write!(f, "Unsupported key type: {:?}", reason)
            }
            CertStoreError::StoreOpenFailed { store_name, error_code } => {
                write!(f, "Failed to open certificate store '{}': error {}", store_name, error_code)
            }
            CertStoreError::StoreCloseFailed { error_code } => {
                write!(f, "Failed to close certificate store: error {}", error_code)
            }
            CertStoreError::ThumbprintNotFound(thumbprint, error_code) => {
                write!(f, "Certificate with thumbprint '{}' not found: error {}", thumbprint, error_code)
            }
            CertStoreError::CertificateNotFound { thumbprint, error_code } => {
                write!(f, "Certificate '{}' not found: error {}", thumbprint, error_code)
            }
            CertStoreError::InvalidThumbprint { thumbprint, reason } => {
                write!(f, "Invalid thumbprint '{}': {}", thumbprint, reason)
            }
            CertStoreError::InvalidStoreHandle => {
                write!(f, "Invalid store handle")
            }
            CertStoreError::CertificateDataMissing => {
                write!(f, "Certificate data is missing")
            }
            CertStoreError::PrivateKeyAcquisitionFailed { error_code } => {
                write!(f, "Failed to acquire private key: error {}", error_code)
            }
            CertStoreError::SigningFailed(reason) => {
                write!(f, "Signing failed: {}", reason)
            }
        }
    }
}

pub type CertResult<T> = std::result::Result<T, CertStoreError>;