use std::ptr;

use winapi::{ctypes::c_void, um::{errhandlingapi::GetLastError, wincrypt::{CERT_CONTEXT, CERT_FIND_ANY, CERT_SHA1_HASH_PROP_ID, CertFindCertificateInStore, CertFreeCertificateContext, CertGetCertificateContextProperty, PKCS_7_ASN_ENCODING, X509_ASN_ENCODING}}};

use crate::cert::{CertContext, CertResult, CertStoreError, Thumbprint};

pub struct CertIterator {
    store: *mut c_void,
    current: *const CERT_CONTEXT,
    started: bool,
}

impl CertIterator {
    pub fn new(store: *mut c_void) -> Self {
        Self {
            store,
            current: ptr::null_mut(),
            started: false,
        }
    }
}

pub struct CertContextIter(*const CERT_CONTEXT);

impl CertContextIter {
    pub unsafe fn from_ptr(ptr: *const CERT_CONTEXT) -> Self {
        Self(ptr)
    }

    pub fn get_thumbprint(&self) -> CertResult<Thumbprint> {
        
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
            
            Thumbprint::from_bytes(thumbprint_bytes)
        }
    }
}

impl Iterator for CertIterator {
    type Item = CertContextIter;
    
    fn next(&mut self) -> Option<Self::Item> {
        unsafe {
            let next = if !self.started {
                self.started = true;
                CertFindCertificateInStore(
                    self.store,
                    X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
                    0,
                    CERT_FIND_ANY,
                    ptr::null(),
                    ptr::null_mut(),
                )
            } else {
                CertFindCertificateInStore(
                    self.store,
                    X509_ASN_ENCODING | PKCS_7_ASN_ENCODING,
                    0,
                    CERT_FIND_ANY,
                    ptr::null(),
                    self.current,
                )
            };
            
            if next.is_null() {
                self.current = ptr::null_mut();
                return None;
            }
            
            self.current = next;
            Some(CertContextIter::from_ptr(next))
        }
    }
}

impl Drop for CertIterator {
    fn drop(&mut self) {
        if !self.current.is_null() {
            unsafe {
                CertFreeCertificateContext(self.current);
            }
        }
    }
}

