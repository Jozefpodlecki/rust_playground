use object::{Object, ObjectSection};
use anyhow::*;

pub struct Loader {

}

impl Loader {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_text_section_data<'a>(&self, data: &'a [u8]) -> Result<&'a [u8]> {
        let obj_file = object::File::parse(&*data)?;

        let text_section = obj_file
            .sections()
            .filter(|pr| pr.name().unwrap() == ".text")
            .next()
            .unwrap();
     
        let data = text_section.data()?;

        Ok(data)
    }
}