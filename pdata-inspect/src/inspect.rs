use std::{
    fs,
    io::{BufWriter, Write},
    path::PathBuf,
};
use anyhow::Result;
use pelite::{pe::{exception, Pe, PeFile}, Wrap};

pub struct Analyser {
    input: PathBuf,
    output: PathBuf,
}

impl Analyser {
    pub fn new(input: PathBuf, output: PathBuf) -> Self {
        Self { input, output }
    }

    pub fn read_pdata(&self) -> Result<()> {
        let data = fs::read(&self.input)?;
        let file = PeFile::from_bytes(&data)?;

        let header = file.optional_header();
        let image_base = header.ImageBase;

        let exception = file.exception().unwrap();

        let mut writer = BufWriter::new(fs::File::create(&self.output)?);

        let func_count = exception.functions().count();

        writeln!(writer, "PE .pdata summary")?;
        writeln!(writer, "=================")?;
        writeln!(writer, "Image base  : 0x{:X}", image_base)?;
        writeln!(writer, "Functions   : {}", func_count)?;
        writeln!(writer)?;

        writeln!(writer, "Function start addresses (VA):")?;

        for function in exception.functions() {
            let va = image_base + function.image().BeginAddress as u64;
            writeln!(writer, "0x{:X}", va)?;
        }

        writer.flush()?;

        println!("Saved to {}",  fs::canonicalize(&self.output).unwrap().display());

        Ok(())
    }
}
