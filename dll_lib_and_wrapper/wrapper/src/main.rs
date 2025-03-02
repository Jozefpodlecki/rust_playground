mod wrapper;
use anyhow::{Result, Ok};
use wrapper::Wrapper;

fn main() -> Result<()> {
    let mut wrapper = Wrapper::new("dll_lib.dll");

    let message = wrapper.get_message().unwrap();
    println!("{:?}", message);

    // let users = wrapper.get_users();
    // println!("{:?}", users);

    // wrapper.set_test_mpsc_with_enum();

    // while let Some(value) = wrapper.recv() {
    //     println!("{:?}", value);
    // }

    Ok(())
}