use std::ptr;

use widestring::U16CString;
use winapi::{ctypes::c_void, um::{errhandlingapi::GetLastError, wincrypt::*}};

use crate::cert::{CertContext, CertIterator, CertResult, CertStoreError, Thumbprint};

pub struct CertStore(*mut c_void);

impl CertStore {
    pub fn open(store_name: &str) -> CertResult<Self> {
        let store_name_wide = U16CString::from_str(store_name)
            .map_err(|_| CertStoreError::StoreOpenFailed {
                store_name: store_name.to_string(),
                error_code: 0,
            })?;
        
        unsafe {
            let handle = CertOpenStore(
                CERT_STORE_PROV_SYSTEM_W,
                0,
                0,
                CERT_SYSTEM_STORE_CURRENT_USER,
                store_name_wide.as_ptr() as *const c_void,
            );
            
            if handle.is_null() {
                let err = GetLastError();
                return Err(CertStoreError::StoreOpenFailed {
                    store_name: store_name.to_string(),
                    error_code: err,
                });
            }
            
            Ok(CertStore(handle))
        }
    }
    
    pub fn open_local_machine(store_name: &str) -> CertResult<Self> {
        let store_name_wide = U16CString::from_str(store_name)
            .map_err(|_| CertStoreError::StoreOpenFailed {
                store_name: store_name.to_string(),
                error_code: 0,
            })?;
        
        unsafe {
            let handle = CertOpenStore(
                CERT_STORE_PROV_SYSTEM_W,
                0,
                0,
                CERT_SYSTEM_STORE_LOCAL_MACHINE,
                store_name_wide.as_ptr() as *const c_void,
            );
            
            if handle.is_null() {
                let err = GetLastError();
                return Err(CertStoreError::StoreOpenFailed {
                    store_name: store_name.to_string(),
                    error_code: err,
                });
            }
            
            Ok(CertStore(handle))
        }
    }
    
    pub fn find_by_thumbprint(&self, thumbprint: &Thumbprint) -> CertResult<CertContext> {
        let thumbprint_bytes = thumbprint.as_bytes();

        let hash_blob = CRYPT_HASH_BLOB {
            cbData: thumbprint_bytes.len() as u32,
            pbData: thumbprint_bytes.as_ptr() as *mut _,
        };

        unsafe {
            let cert = CertFindCertificateInStore(
                self.0,
                X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
                0,
                CERT_FIND_SHA1_HASH,
                &hash_blob as *const _ as *const _,
                ptr::null_mut(),
            );
         
            if cert.is_null() {
                let err = GetLastError();
                return Err(CertStoreError::ThumbprintNotFound(*thumbprint, err));
            }
            
            Ok(CertContext::from_ptr(cert))
        }
    }
    
    pub fn iter(&self) -> CertIterator {
        CertIterator::new(self.0)
    }
    
    pub fn as_ptr(&self) -> *mut c_void {
        self.0
    }
}

impl Drop for CertStore {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                CertCloseStore(self.0, 0);
            }
        }
    }
}

