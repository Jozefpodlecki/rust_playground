mod wrapper;
mod tokio_mpsc_wrapper;
use anyhow::{Result, Ok};
use tokio_mpsc_wrapper::TokioMpscWrapper;
use wrapper::Wrapper;

#[tokio::main]
async fn main() -> Result<()> {

    let mut wrapper = TokioMpscWrapper::new("dll_lib.dll");

    wrapper.load();

    while let Some(value) = wrapper.recv().await {
        println!("{:?}", value);
    }

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