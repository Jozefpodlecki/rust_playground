#![allow(warnings)]

use anyhow::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Test<'a> {
    value: &'a str
}

fn main() -> Result<()> {

    let value = Box::new("{ \"value\": \"test\" }");

    let obj: Test = serde_json::from_str(*value)?;

    println!("{:?}", value.as_ptr());
    println!("{:?}", obj.value.as_ptr());
    println!("{:?}", obj.value.as_ptr() as usize - value.as_ptr() as usize);
    println!("{:?}", value.find("test").unwrap());


    Ok(())
}