use object::{File, Object, ObjectSection, Section};
use anyhow::*;

pub struct Loader<'a>(File<'a>);

impl<'a> Loader<'a> {
    pub fn new(data: &'a [u8]) -> Result<Self> {
        let obj_file = object::File::parse(&*data)?;
        Ok(Self(obj_file))
    }

    pub fn get_text_section(&self) -> Result<Section<'a, '_>> {
        
        let text_section = self.0
            .sections()
            .filter(|pr| pr.name().unwrap() == ".text")
            .next()
            .unwrap();

        Ok(text_section)
    }
}