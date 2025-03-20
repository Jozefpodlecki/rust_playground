mod wrapper;
mod trait_wrapper;
mod tokio_mpsc_wrapper;
use anyhow::{Result, Ok};
use tokio_mpsc_wrapper::TokioMpscWrapper;
use trait_wrapper::TraitWrapper;
use wrapper::Wrapper;

#[tokio::main]
async fn main() -> Result<()> {

    let mut wrapper = TraitWrapper::new("dll_lib.dll");

    let mut service = wrapper.get_service_raw_pin();
    // let service = service.as_mut();
    // service.start().unwrap();

    // let wrapped_service = wrapper.get_service_wrapped();
    // let wrapped_service = unsafe { Box::from_raw(wrapped_service) };
    // let wrapped_service = unsafe { &mut * wrapped_service };
    // let mut service = wrapped_service.service;

    // println!("start");
    // println!("{}", wrapped_service.version);
    // wrapped_service.service.start().unwrap();
    // service.start().unwrap();
    // service.stop().unwrap();

    // let mut wrapper = TokioMpscWrapper::new("dll_lib.dll");

    // wrapper.load();

    // while let Some(value) = wrapper.recv().await {
    //     println!("{:?}", value);
    // }

    // let mut wrapper = Wrapper::new("dll_lib.dll");

    // let message = wrapper.get_message().unwrap();
    // println!("{:?}", message);

    // let users = wrapper.get_users();
    // println!("{:?}", users);

    // wrapper.set_test_mpsc_with_enum();

    // while let Some(value) = wrapper.recv() {
    //     println!("{:?}", value);
    // }

    Ok(())
}