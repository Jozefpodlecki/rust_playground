use std::ops::{Deref, DerefMut};

use abi_stable::{*, library::RootModule, sabi_types::VersionStrings, std_types::{RBox, RBoxError, RResult}, StableAbi};
use abi_stable::external_types::crossbeam_channel::RReceiver;
use models::Command;
use tokio::sync::mpsc::UnboundedReceiver;

pub mod models;
pub mod traits;

#[sabi_trait]
pub trait Service {
    fn start(&mut self) -> RResult<RReceiver<i64> , RBoxError>;
    fn stop(&mut self) -> RResult<(), RBoxError>;
}

pub type ServiceType = Service_TO<'static, RBox<()>>;

#[derive(StableAbi)]
#[repr(C)]
pub struct TokioMpscWrapper(*mut ());

impl TokioMpscWrapper {
    pub fn new(rx: UnboundedReceiver<Command>) -> Self {
        Self(Box::into_raw(Box::new(rx)) as *mut ())
    }
}

impl Deref for TokioMpscWrapper {
    type Target = UnboundedReceiver<Command>;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.0 as *const Self::Target) }
    }
}

impl DerefMut for TokioMpscWrapper {

    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.0 as *mut Self::Target) }
    }
}

#[sabi_trait]
pub trait TokioService {
    fn start_v2(&mut self) -> TokioMpscWrapper;
    fn start(&mut self) -> *mut ();
    fn stop(&mut self) -> RResult<(), RBoxError>;
}

pub type TokioServiceType = TokioService_TO<'static, RBox<()>>;


#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = ServiceRoot_Ref)))]
#[sabi(missing_field(panic))]
pub struct ServiceRoot {
    #[sabi(last_prefix_field)]
    pub new: extern "C" fn() -> RResult<ServiceType, RBoxError>,

    #[sabi(last_prefix_field)]
    pub new_tokio: extern "C" fn() -> RResult<TokioServiceType, RBoxError>,
}

impl RootModule for ServiceRoot_Ref {
    declare_root_module_statics! {ServiceRoot_Ref}
    const BASE_NAME: &'static str = "serviceroot";
    const NAME: &'static str = "serviceroot";
    const VERSION_STRINGS: VersionStrings = package_version_strings!();
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix(prefix_ref = TestMod_Ref)))]
#[sabi(missing_field(panic))]
pub struct TestMod {
    pub something: std::marker::PhantomData<()>,
}