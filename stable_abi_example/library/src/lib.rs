use abi_stable::{export_root_module, sabi_extern_fn, sabi_trait::TD_Opaque, std_types::{RBoxError, RResult::{self, ROk}}};
use shared::{Service, ServiceRoot, ServiceRoot_Prefix, ServiceRoot_Ref, ServiceType, Service_TO};
use abi_stable::prefix_type::PrefixTypeTrait;

#[export_root_module]
fn instantiate_root_module() -> ServiceRoot_Ref {
    ServiceRoot { new }.leak_into_prefix()
}

#[sabi_extern_fn]
pub fn new() -> RResult<ServiceType, RBoxError> {
    let this = PrintService {  };
    ROk(Service_TO::from_value(this, TD_Opaque))
}

struct PrintService {

}

impl Service for PrintService {
    
    fn start(&mut self) {
        println!("test");
    }
}