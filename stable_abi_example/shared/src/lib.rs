use abi_stable::{*, library::RootModule, sabi_types::VersionStrings, std_types::{RBox, RBoxError, RResult}, StableAbi};
use abi_stable::external_types::crossbeam_channel::RReceiver;

#[sabi_trait]
pub trait Service {
    fn start(&mut self) -> RResult<RReceiver<i64> , RBoxError>;
    fn stop(&mut self) -> RResult<(), RBoxError>;
}

pub type ServiceType = Service_TO<'static, RBox<()>>;

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = ServiceRoot_Ref)))]
#[sabi(missing_field(panic))]
pub struct ServiceRoot {
    #[sabi(last_prefix_field)]
    pub new: extern "C" fn() -> RResult<ServiceType, RBoxError>,
}

impl RootModule for ServiceRoot_Ref {
    declare_root_module_statics! {ServiceRoot_Ref}
    const BASE_NAME: &'static str = "serviceroot";
    const NAME: &'static str = "serviceroot";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

