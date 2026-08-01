
#![feature(mpmc_channel)]
#![feature(unsafe_cell_access)]
#![feature(generic_atomic)]
#![allow(unused)]
#![allow(static_mut_refs)]

use core::any::TypeId;
use core::ptr;
use core::mem;
use mkb_raw_input::*;
use core::any::Any;

mod cert;

use sha2::{Digest, Sha256};
use crate::cert::*;

fn run() -> Result<(), Box<dyn std::error::Error>> {
    let store = CertStore::open("WDRTestCertStore")?;
 
    let mut count = 0;
    for cert in store.iter() {
        count += 1;
        let tp = cert.get_thumbprint()?;
        println!("  {}. {}", count, tp);
    }
 
    let thumbprint = Thumbprint::new("1998B1435409E2B7316004BB2919814C1200AA96")?;
    let cert = store.find_by_thumbprint(&thumbprint)?;   
    let data = cert.get_encoded_data()?;
    let tp = cert.get_thumbprint()?;
    let key = cert.private_key()?;
    let info = key.info()?;
    println!("{info}");
    let data = b"test";
    let hash = Sha256::digest(data);
    let signature = key.sign_fixed::<256>(&hash)?;
    println!("{signature:?}");

    Ok(())
}

fn main() {
    if let Err(err) = run() {
        println!("{err}");
    }
}
