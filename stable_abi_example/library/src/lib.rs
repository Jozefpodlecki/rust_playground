mod background_service;
mod tokio_background_service;

use std::sync::{atomic::AtomicBool, Arc};

use abi_stable::{export_root_module, sabi_extern_fn, sabi_trait::TD_Opaque, std_types::{RBoxError, RResult::{self, ROk}}};
use background_service::BackgroundService;
use shared::{Service, ServiceRoot, ServiceRoot_Prefix, ServiceRoot_Ref, ServiceType, Service_TO, TokioServiceType, TokioService_TO};
use abi_stable::prefix_type::PrefixTypeTrait;
use tokio_background_service::TokioBackgroundService;

#[export_root_module]
fn instantiate_root_module() -> ServiceRoot_Ref {
    ServiceRoot { 
        new,
        new_tokio  
    }.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn new_tokio() -> RResult<TokioServiceType, RBoxError> {
    let this = TokioBackgroundService { 
        tx: None,
        handle: None,
        close_flag: Arc::new(AtomicBool::new(false))
     };
    ROk(TokioService_TO::from_value(this, TD_Opaque))
}

#[sabi_extern_fn]
pub fn new() -> RResult<ServiceType, RBoxError> {
    let this = BackgroundService { 
        tx: None,
        handle: None,
        close_flag: Arc::new(AtomicBool::new(false))
     };
    ROk(Service_TO::from_value(this, TD_Opaque))
}

// #[sabi_extern_fn]
// pub fn deserialize_state(s: RStr<'_>) -> RResult<TOStateBox, RBoxError> {
//     deserialize_json::<TextOperationState>(s).map(DynTrait::from_value)
// }