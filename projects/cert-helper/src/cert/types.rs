use core::fmt;

use crate::cert::CertResult;
use crate::cert::CertStoreError;
use crate::cert::PrivateKey;
use winapi::um::wincrypt::*;
use winapi::um::errhandlingapi::GetLastError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Thumbprint([u8; 20]);

impl Thumbprint {
    pub fn from_bytes(data: [u8; 20]) -> CertResult<Self> {
        Ok(Self(data))
    }

    pub fn new(data: &str) -> CertResult<Self> {

        let data = data.trim();
        if data.len() != 40 {
            return Err(CertStoreError::InvalidThumbprint {
                thumbprint: data.to_string(),
                reason: "must be 40 hex characters".to_string(),
            });
        }

        let mut buffer: [u8; 20] = [0; 20];
        
        hex::decode_to_slice(data, &mut buffer).map_err(|_| CertStoreError::InvalidThumbprint {
            thumbprint: data.to_string(),
            reason: "invalid hex characters".to_string(),
        })?;

        Ok(Thumbprint(buffer))
    }
    
    pub fn as_bytes(&self) -> &[u8; 20] {
        &self.0
    }
    
    pub fn to_hex_string(&self) -> String {
        hex::encode(self.0).to_uppercase()
    }
}

impl fmt::Display for Thumbprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", hex::encode(self.0).to_uppercase())
    }
}

#[derive(Debug, Clone)]
pub struct CertData(Vec<u8>);

impl CertData {
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
    
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

pub struct CertContext(*const CERT_CONTEXT);

impl CertContext {
    pub unsafe fn from_ptr(ptr: *const CERT_CONTEXT) -> Self {
        Self(ptr)
    }
    
    pub fn as_ptr(&self) -> *const CERT_CONTEXT {
        self.0
    }
    
    pub fn is_null(&self) -> bool {
        self.0.is_null()
    }

    pub fn private_key(&self) -> CertResult<PrivateKey> {
        PrivateKey::from_cert_context(self)
    }
    
    pub fn get_encoded_data(&self) -> CertResult<CertData> {
        if self.is_null() {
            return Err(CertStoreError::InvalidStoreHandle);
        }
        
        unsafe {
            let cert_info = &(*self.0);
            let data = cert_info.pbCertEncoded;
            let len = cert_info.cbCertEncoded as usize;
            
            if data.is_null() || len == 0 {
                return Err(CertStoreError::CertificateDataMissing);
            }
            
            Ok(CertData(std::slice::from_raw_parts(data, len).to_vec()))
        }
    }
    
    pub fn get_thumbprint(&self) -> CertResult<Thumbprint> {
        if self.is_null() {
            return Err(CertStoreError::InvalidStoreHandle);
        }
        
        unsafe {
            let mut thumbprint_bytes = [0u8; 20];
            let mut thumbprint_len = 20;
            let result = CertGetCertificateContextProperty(
                self.0,
                CERT_SHA1_HASH_PROP_ID,
                thumbprint_bytes.as_mut_ptr() as *mut _,
                &mut thumbprint_len,
            );
            
            if result == 0 {
                let err = GetLastError();
                return Err(CertStoreError::CertificateNotFound {
                    thumbprint: "unknown".to_string(),
                    error_code: err,
                });
            }
            
            Ok(Thumbprint(thumbprint_bytes))
        }
    }
}

impl Drop for CertContext {
    fn drop(&mut self) {
        if !self.is_null() {
            unsafe {
                CertFreeCertificateContext(self.0);
            }
        }
    }
}
