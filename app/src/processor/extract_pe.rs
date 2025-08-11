use std::{collections::HashMap, fs::{self, File}, io::{Cursor, Write}, path::PathBuf};
use anyhow::Result;
use log::*;
use object::{read::pe::PeFile64, LittleEndian, Object, ObjectSection};
use serde::Serialize;
use crate::{disassembler::Disassembler, processor::ProcessorStep};

pub struct ExtractPeStep {
    exe_path: PathBuf,
    dest_path: PathBuf,
}

#[derive(Serialize)]
struct PeSummary {
    file_name: String,
    entry_point_rva: String,
    entry_point_va: String,
    image_base: String,
    sections: Vec<SectionSummary>,
}

#[derive(Serialize)]
struct SectionSummary {
    name: String,
    address: String,
    size: u64,
}

impl ProcessorStep for ExtractPeStep {
    fn name(&self) -> String {
        format!("Extracting PE {:?}", self.exe_path.file_name().unwrap())
    }

    fn can_execute(&self) -> bool {
      

        true
    }

    fn execute(self: Box<Self>) -> Result<()> {

        let file_name = self.exe_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();
        let dest_path = self.dest_path.join(&file_name).join("PE");

        if !dest_path.exists() {
            fs::create_dir_all(&dest_path)?;
        }

        let data = fs::read(self.exe_path)?;

        let pe_file = PeFile64::parse(&*data)?;

        let opt_header = pe_file.nt_headers();
        let image_base = opt_header.optional_header.image_base.get(LittleEndian);
        let address_of_entry_point_rva = opt_header.optional_header.address_of_entry_point.get(LittleEndian);
        let address_of_entry_point = image_base + address_of_entry_point_rva as u64;

        let mut summary = PeSummary {
            file_name: file_name.clone(),
            entry_point_rva: format!("0x{:X}", address_of_entry_point_rva),
            entry_point_va: format!("0x{:X}", address_of_entry_point),
            image_base: format!("0x{:X}", image_base),
            sections: Vec::new(),
        };

        for section in pe_file.sections() {
            let sec_name_bytes = section.name_bytes()?;
            let sec_name = match std::str::from_utf8(sec_name_bytes) {
                Ok(str) if str.trim().is_empty() => {
                    hex::encode(sec_name_bytes)
                },
                Ok(str) => str.trim().to_string(),
                Err(_) => hex::encode(sec_name_bytes),
            };
            let address = section.address();
            let size = section.size();
            let file_name = format!("0x{:X}_{}_{}.section", address, size, sec_name);
            let file_path = dest_path.join(&file_name);

            if !file_path.exists() {
                info!("Saving {:?}", file_path.strip_prefix(&dest_path)?);
                let data = section.data()?;
                File::create(&file_path)?.write_all(data)?;
            } else {
                info!("Skipping existing section {}", file_name);
            }

            if address_of_entry_point == address {
                let disassembler = Disassembler::new()?;
                let data = section.data()?;
                let reader = Cursor::new(data);
                let file_name = format!("0x{:X}_{}_{}.txt", address, size, sec_name);
                let file_path = dest_path.join(&file_name);
                disassembler.export_to_txt(address_of_entry_point, reader, &file_path)?;
            }

            summary.sections.push(SectionSummary {
                name: sec_name,
                address: format!("0x{:X}", address),
                size: section.size(),
            });
        }

        let summary_path = dest_path.join("summary.json");
        let writer = File::create(summary_path)?;
        serde_json::to_writer_pretty(writer, &summary)?;

        Ok(())
    }
}

impl ExtractPeStep {
    pub fn new(
        exe_path: PathBuf,
        dest_path: PathBuf
    ) -> Self {

        Self {
            exe_path,
            dest_path,
        }
    }
}