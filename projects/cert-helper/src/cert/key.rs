use core::fmt;
use std::ptr;

use widestring::{U16CString, U16Str, U16String};
use winapi::shared::bcrypt::{BCRYPT_PAD_PKCS1, BCRYPT_PKCS1_PADDING_INFO, BCRYPT_SHA256_ALGORITHM};
use winapi::shared::minwindef::{DWORD, PBYTE};
use winapi::shared::ntdef::{LPCWSTR, SECURITY_STATUS};
use winapi::um::ncrypt::{NCRYPT_KEY_HANDLE, NCRYPT_PAD_PKCS1_FLAG};
use winapi::um::wincrypt::*;
use winapi::um::errhandlingapi::GetLastError;
use winapi::ctypes::c_void;
use crate::cert::{SigningFailedReason, UnsupportedKeyTypeReason};
use crate::cert::error::{CertStoreError, CertResult};
use crate::cert::types::CertContext;

#[link(name = "ncrypt")]
unsafe extern "system" {
    pub fn NCryptSignHash(
        hKey: NCRYPT_KEY_HANDLE,
        pPaddingInfo: *mut c_void,
        pbHashValue: PBYTE,
        cbHashValue: DWORD,
        pbSignature: PBYTE,
        cbSignature: DWORD,
        pcbResult: *mut DWORD,
        dwFlags: DWORD,
    ) -> SECURITY_STATUS;

    pub fn NCryptGetProperty(
        hObject: NCRYPT_KEY_HANDLE,
        pszProperty: LPCWSTR,
        pbOutput: PBYTE,
        cbOutput: DWORD,
        pcbResult: *mut DWORD,
        dwFlags: DWORD,
    ) -> SECURITY_STATUS;
}

#[derive(Debug, Clone)]
pub struct KeyInfo {
    pub algorithm: String,
    pub key_size: u32,
    pub key_type: KeyType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum KeyType {
    Rsa,
    Ecdsa,
    Ecdh,
    Unknown,
}

impl fmt::Display for KeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KeyType::Rsa => write!(f, "RSA"),
            KeyType::Ecdsa => write!(f, "ECDSA"),
            KeyType::Ecdh => write!(f, "ECDH"),
            KeyType::Unknown => write!(f, "Unknown"),
        }
    }
}

impl fmt::Display for KeyInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Algorithm: {}, Key Size: {} bits, Type: {}",
            self.algorithm, self.key_size, self.key_type
        )
    }
}

#[derive(Debug)]
pub struct PrivateKey {
    handle: HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
    key_spec: u32,
    must_free: i32,
    is_cng: bool,
}

impl PrivateKey {
    pub fn from_cert_context(cert: &CertContext) -> CertResult<Self> {
        let mut handle: HCRYPTPROV_OR_NCRYPT_KEY_HANDLE = 0;
        let mut key_spec: u32 = 0;
        let mut must_free: i32 = 0;
        
        let result = unsafe {
            CryptAcquireCertificatePrivateKey(
                cert.as_ptr(),
                CRYPT_ACQUIRE_SILENT_FLAG | CRYPT_ACQUIRE_PREFER_NCRYPT_KEY_FLAG,
                std::ptr::null_mut(),
                &mut handle,
                &mut key_spec,
                &mut must_free,
            )
        };
        
        if result == 0 {
            let err = unsafe { GetLastError() };
            return Err(CertStoreError::PrivateKeyAcquisitionFailed { error_code: err });
        }
        
        let is_cng = key_spec == CERT_NCRYPT_KEY_SPEC;
        
        Ok(PrivateKey {
            handle,
            key_spec,
            must_free,
            is_cng,
        })
    }
    
    pub fn handle(&self) -> HCRYPTPROV_OR_NCRYPT_KEY_HANDLE {
        self.handle
    }
    
    pub fn is_cng(&self) -> bool {
        self.is_cng
    }
    
    pub fn key_spec(&self) -> u32 {
        self.key_spec
    }

    pub fn info(&self) -> CertResult<KeyInfo> {
        if !self.is_cng {
            return Err(CertStoreError::UnsupportedKeyType(UnsupportedKeyTypeReason::NotCng));
        }
        
        let alg_name = U16CString::from_str("Algorithm Name")
            .map_err(|_| CertStoreError::SigningFailed(SigningFailedReason::InvalidParameter))?;
        
        let length_name = U16CString::from_str("Length")
            .map_err(|_| CertStoreError::SigningFailed(SigningFailedReason::InvalidParameter))?;
        
        unsafe {
            let mut alg_len: DWORD = 0;
            let status = NCryptGetProperty(
                self.handle,
                alg_name.as_ptr() as LPCWSTR,
                ptr::null_mut(),
                0,
                &mut alg_len,
                0,
            );
            
            let algorithm = if status == 0 && alg_len > 0 {
                let mut alg_buf = vec![0u16; (alg_len / 2) as usize];
                let mut result_len: DWORD = 0;
                let status = NCryptGetProperty(
                    self.handle,
                    alg_name.as_ptr() as LPCWSTR,
                    alg_buf.as_mut_ptr() as *mut u8,
                    alg_len,
                    &mut result_len,
                    0,
                );
                
                if status == 0 {
                    U16String::from_vec(alg_buf).to_string_lossy()
                } else {
                    "Unknown".to_string()
                }
            } else {
                "Unknown".to_string()
            };
            
            let mut key_size: u32 = 0;
            let mut result_len: DWORD = 0;
            let status = NCryptGetProperty(
                self.handle,
                length_name.as_ptr() as LPCWSTR,
                (&mut key_size as *mut u32) as *mut u8,
                std::mem::size_of::<u32>() as DWORD,
                &mut result_len,
                0,
            );
            
            let key_type = if algorithm.contains("RSA") {
                KeyType::Rsa
            } else if algorithm.contains("ECDSA") {
                KeyType::Ecdsa
            } else if algorithm.contains("ECDH") {
                KeyType::Ecdh
            } else {
                KeyType::Unknown
            };
            
            Ok(KeyInfo {
                algorithm,
                key_size,
                key_type,
            })
        }
    }

    pub fn sign_fixed<const N: usize>(&self, hash: &[u8]) -> CertResult<[u8; N]> {
        if !self.is_cng {
            return Err(CertStoreError::UnsupportedKeyType(UnsupportedKeyTypeReason::NotCng));
        }
        
        let sha256_wide = U16String::from_str("SHA256");
        let mut padding_info = BCRYPT_PKCS1_PADDING_INFO {
            pszAlgId: sha256_wide.as_ptr() as *mut u16,
        };
        
        let mut signature_len: DWORD = 0;
        
        let status = unsafe {
            NCryptSignHash(
                self.handle,
                &mut padding_info as *mut _ as *mut c_void,
                hash.as_ptr() as *mut u8,
                hash.len() as DWORD,
                ptr::null_mut(),
                0,
                &mut signature_len,
                NCRYPT_PAD_PKCS1_FLAG,
            )
        };
        
        if status != 0 {
            let reason = SigningFailedReason::from(status);
            return Err(CertStoreError::SigningFailed(reason));
        }
        
        if signature_len as usize != N {
            return Err(CertStoreError::SigningFailed(SigningFailedReason::InvalidParameter));
        }
        
        let mut signature = [0u8; N];
        let status = unsafe {
            NCryptSignHash(
                self.handle,
                &mut padding_info as *mut _ as *mut c_void,
                hash.as_ptr() as *mut u8,
                hash.len() as DWORD,
                signature.as_mut_ptr(),
                signature_len,
                &mut signature_len,
                NCRYPT_PAD_PKCS1_FLAG,
            )
        };
        
        if status != 0 {
            let reason = SigningFailedReason::from(status);
            return Err(CertStoreError::SigningFailed(reason));
        }
        
        Ok(signature)
    }

    pub fn sign(&self, hash: &[u8]) -> CertResult<Vec<u8>> {
        if !self.is_cng {
            return Err(CertStoreError::UnsupportedKeyType(UnsupportedKeyTypeReason::NotCng));
        }
        
        let sha256_wide = U16String::from_str("SHA256");
        
        let mut padding_info = BCRYPT_PKCS1_PADDING_INFO {
            pszAlgId: sha256_wide.as_ptr() as *mut u16,
        };
            
        let mut signature_len: DWORD = 0;
        
        let status = unsafe {
            NCryptSignHash(
                self.handle,
                &mut padding_info as *mut _ as *mut c_void,
                hash.as_ptr() as *mut u8,
                hash.len() as DWORD,
                ptr::null_mut(),
                0,
                &mut signature_len,
                NCRYPT_PAD_PKCS1_FLAG,
            )
        };
        
        if status != 0 {
            let reason = SigningFailedReason::from(status);
            return Err(CertStoreError::SigningFailed(reason));
        }
        // println!("signature_len {signature_len}");
        let mut signature = vec![0u8; signature_len as usize];
        
        let status = unsafe {
            NCryptSignHash(
                self.handle,
                &mut padding_info as *mut _ as *mut c_void,
                hash.as_ptr() as *mut u8,
                hash.len() as DWORD,
                signature.as_mut_ptr(),
                signature_len,
                &mut signature_len,
                NCRYPT_PAD_PKCS1_FLAG,
            )
        };
        
        if status != 0 {
             let reason = SigningFailedReason::from(status);
            return Err(CertStoreError::SigningFailed(reason));
        }
        
        signature.truncate(signature_len as usize);
        Ok(signature)
    }
}

impl Drop for PrivateKey {
    fn drop(&mut self) {
        if self.must_free != 0 {
            unsafe {
                if self.is_cng {
                    use winapi::um::ncrypt::NCryptFreeObject;
                    NCryptFreeObject(self.handle);
                } else {
                    CryptReleaseContext(self.handle, 0);
                }
            }
        }
    }
}