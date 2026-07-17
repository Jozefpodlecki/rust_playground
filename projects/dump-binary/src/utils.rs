use alloc::{collections::btree_map::BTreeMap, string::{String, ToString}, vec::Vec};
use pelite::{image::*, pe::{Pe, PeFile, PeObject, image::*}};
use toolkit::*;

use crate::types::*;

fn parse_version(major: u16, minor: u16) -> Version {
    Version { major, minor }
}

fn parse_version_from_u16(value: u16) -> Version {
    Version {
        major: value >> 8,
        minor: value & 0xFF,
    }
}

fn parse_data_directory(pe: &PeFile<'_>) -> DataDirectory {
    let dirs = pe.data_directory();
    let mut directories = BTreeMap::new();
    for (i, dir) in dirs.iter().enumerate() {
        if dir.VirtualAddress != 0 || dir.Size != 0 {
            let dir_type: DataDirectoryType = i.into();
            directories.insert(dir_type, DataDirectoryEntry {
                directory_type: dir_type,
                virtual_address: dir.VirtualAddress.into(),
                size: dir.Size.into(),
            });
        }
    }
    DataDirectory(directories)
}

fn parse_rich_header(pe: &PeFile<'_>) -> Option<RichHeader> {
    if let Ok(rich_info) = pe.rich_structure() {
        let mut records = Vec::new();
        for record in rich_info.records() {
            records.push(RichRecord {
                id: record.product,
                version: record.build,
                count: record.count,
            });
        }
        Some(RichHeader {
            image_checksum: rich_info.checksum(),
            xor_key: rich_info.xor_key(),
            records,
        })
    } else {
        None
    }
}

fn parse_exports(pe: &PeFile<'_>) -> Option<ExportDirectory> {
    let exports = match pe.exports() {
        Ok(value) => value,
        Err(_) => return None,
    };
    
    let by = match exports.by() {
        Ok(value) => value,
        Err(_) => return None,
    };

    let mut entries = Vec::new();
        
    for (name_result, export_result) in by.iter_names() {
        if let (Ok(name), Ok(export)) = (name_result, export_result) {
            let name_str = name.to_str().unwrap_or("").to_string();
            let rva = match export {
                pelite::Export::Symbol(rva) => *rva,
                pelite::Export::Forward(_) => 0,
            };
            
            entries.push(ExportEntry {
                name: name_str,
                ordinal: 0,
                rva: rva.into(),
            });
        }
    }
    
    for (index, export_result) in by.iter().enumerate() {
        if let Ok(export) = export_result {
            if let Ok(import) = by.name_lookup(index) {
                let ord = match import {
                    pelite::Import::ByOrdinal { ord } => ord,
                    _ => 0,
                };

                if !entries.iter().any(|e| e.ordinal == ord) {
                    let rva = match export {
                        pelite::Export::Symbol(rva) => *rva,
                        pelite::Export::Forward(_) => 0,
                    };
                    entries.push(ExportEntry {
                        name: format!("ordinal_{}", ord),
                        ordinal: ord,
                        rva: rva.into(),
                    });
                }
            }
        }
    }
    
    return Some(ExportDirectory(entries));
}


fn parse_imports(pe: &PeFile<'_>) -> Option<ImportDirectory> {
    let imports = match pe.imports() {
        Ok(value) => value,
        Err(_) => return None,
    };

    let mut modules = Vec::new();
    for desc in imports {
        let dll_name = desc.dll_name()
            .ok()
            .and_then(|cstr| cstr.to_str().ok())
            .map(|s| s.to_string())
            .unwrap_or_default();
        
        let mut functions = Vec::new();
        
        if let Ok(int_iter) = desc.int() {
            for import_result in int_iter {
                if let Ok(import) = import_result {
                    match import {
                        pelite::Import::ByName { hint, name } => {
                            let name_str = name.to_str().unwrap_or("").to_string();
                            functions.push(ImportFunction {
                                name: name_str,
                                hint,
                                ordinal: 0,
                                rva: 0u32.into(),
                            });
                        }
                        pelite::Import::ByOrdinal { ord } => {
                            functions.push(ImportFunction {
                                name: format!("ordinal_{}", ord),
                                hint: 0,
                                ordinal: ord,
                                rva: 0u32.into(),
                            });
                        }
                    }
                }
            }
        }
        
        if let Ok(iat_iter) = desc.iat() {
            for (i, &va) in iat_iter.enumerate() {
                if let Some(import) = import_from_va(*pe, va) {
                    match import {
                        pelite::Import::ByName { hint, name } => {
                            let name_str = name.to_str().unwrap_or("").to_string();
                            if let Some(existing) = functions.iter_mut().find(|f| f.name == name_str) {
                                existing.rva = va.into();
                            }
                        }
                        pelite::Import::ByOrdinal { ord } => {
                            if let Some(existing) = functions.iter_mut().find(|f| f.ordinal == ord) {
                                existing.rva = va.into();
                            }
                        }
                    }
                }
            }
        }
        
        modules.push(ImportModule {
            dll_name,
            functions,
        });
    }

    Some(ImportDirectory(modules))
}

fn import_from_va<'a, P: Pe<'a>>(pe: P, va: Va) -> Option<pelite::Import<'a>> {
    if va & IMAGE_ORDINAL_FLAG == 0 {
        let rva = va as Rva;
        let hint = pe.derva::<u16>(rva).ok()?;
        let name = pe.derva_c_str(rva + 2).ok()?;
        Some(pelite::Import::ByName { hint: *hint as usize, name })
    } else {
        Some(pelite::Import::ByOrdinal { ord: va as Ordinal })
    }
}

fn parse_relocs(pe: &PeFile<'_>) -> Option<RelocationDirectory> {
    let relocs = match pe.base_relocs() {
        Ok(value) => value,
        Err(_) => return None,
    };
    
    let mut blocks = Vec::new();
    for block in relocs.iter_blocks() {
        let image = block.image();
        let mut entries = Vec::new();
        
        for word in block.words() {
            let ty = block.type_of(word);
            if ty != IMAGE_REL_BASED_ABSOLUTE {
                let rva = block.rva_of(word);
                entries.push(RelocationEntry {
                    rva: rva.into(),
                    r#type: ty,
                });
            }
        }
        
        if !entries.is_empty() {
            blocks.push(RelocationBlock {
                virtual_address: image.VirtualAddress.into(),
                entries,
            });
        }
    }
    
    Some(RelocationDirectory(blocks))
}

fn parse_exception_directory(pe: &PeFile<'_>) -> Option<ExceptionDirectory> {
    let exception = match pe.exception() {
        Ok(value) => value,
        Err(_) => return None,
    };

    let mut entries = Vec::new();
    for function in exception.functions() {
        let image = function.image();
        entries.push(ExceptionEntry {
            begin_rva: image.BeginAddress.into(),
            end_rva: image.EndAddress.into(),
            unwind_rva: image.UnwindData.into(),
        });
    }

    Some(ExceptionDirectory(entries))
}

fn parse_section_data(pe: &PeFile<'_>, section: &pelite::image::IMAGE_SECTION_HEADER) -> SectionData {
    if section.SizeOfRawData == 0 || section.PointerToRawData == 0 {
        return SectionData::default();
    }

    let start = section.PointerToRawData as usize;
    let end = start + section.SizeOfRawData as usize;
    let file_bytes = pe.image();
    
    if end > file_bytes.len() {
        return SectionData::default();
    }

    let bytes = &file_bytes[start..end];
    SectionData::from_bytes(bytes)
}

impl Binary {
    pub fn new(file: PeFile<'_>, options: &SerializedBinaryOptions) -> Result<Self, &'static str> {
        let nt = file.nt_headers();
        let file_header = &nt.FileHeader;
        let optional = nt.OptionalHeader;

        let file_header_struct = FileHeader {
            machine: file_header.Machine.into(),
            number_of_sections: file_header.NumberOfSections.into(),
            time_date_stamp: file_header.TimeDateStamp.into(),
            pointer_to_symbol_table: file_header.PointerToSymbolTable.into(),
            number_of_symbols: file_header.NumberOfSymbols.into(),
            size_of_optional_header: file_header.SizeOfOptionalHeader.into(),
            characteristics: file_header.Characteristics.into()
        };
        
        let optional_header_struct = OptionalHeader {
            magic: optional.Magic,
            linker_version: format!("{}.{}", optional.LinkerVersion.Major, optional.LinkerVersion.Minor),
            size_of_code: optional.SizeOfCode.into(),
            size_of_initialized_data: optional.SizeOfInitializedData.into(),
            size_of_uninitialized_data: optional.SizeOfUninitializedData.into(),
            address_of_entry_point: optional.AddressOfEntryPoint.into(),
            base_of_code: optional.BaseOfCode,
            image_base: optional.ImageBase.into(),
            section_alignment: optional.SectionAlignment.into(),
            file_alignment: optional.FileAlignment.into(),
            operating_system_version: Version { 
                major: optional.OperatingSystemVersion.Major, 
                minor: optional.OperatingSystemVersion.Minor 
            },
            image_version: Version { 
                major: optional.ImageVersion.Major, 
                minor: optional.ImageVersion.Minor 
            },
            subsystem_version: Version { 
                major: optional.SubsystemVersion.Major, 
                minor: optional.SubsystemVersion.Minor,
            },
            win32_version_value: optional.Win32VersionValue,
            size_of_image: optional.SizeOfImage.into(),
            size_of_headers: optional.SizeOfHeaders.into(),
            check_sum: optional.CheckSum,
            subsystem: optional.Subsystem.into(),
            dll_characteristics: optional.DllCharacteristics.into(),
            size_of_stack_reserve: optional.SizeOfStackReserve.into(),
            size_of_stack_commit: optional.SizeOfStackCommit.into(),
            size_of_heap_reserve: optional.SizeOfHeapReserve.into(),
            size_of_heap_commit: optional.SizeOfHeapCommit.into(),
            loader_flags: optional.LoaderFlags.into(),
            number_of_rva_and_sizes: optional.NumberOfRvaAndSizes.into(),
        };
        
        let data_directory = if options.include_data_directories {
            parse_data_directory(&file)
        } else {
            DataDirectory::default()
        };
        
        let mut sections = Vec::new();
        for section in file.section_headers() {
            let name_bytes = &section.Name;
            let section_data = if options.include_section_data {
                parse_section_data(&file, section)
            } else {
                SectionData::default()
            };
            
            sections.push(Section {
                name: SectionName::from_bytes(name_bytes),
                data: section_data,
                virtual_size: section.VirtualSize.into(),
                virtual_address: section.VirtualAddress.into(),
                size_of_raw_data: section.SizeOfRawData.into(),
                pointer_to_raw_data: section.PointerToRawData.into(),
                pointer_to_relocations: section.PointerToRelocations.into(),
                pointer_to_linenumbers: section.PointerToLinenumbers.into(),
                number_of_relocations: section.NumberOfRelocations.into(),
                number_of_linenumbers: section.NumberOfLinenumbers.into(),
                characteristics: section.Characteristics.into(),
            });
        }
        
        let rich_header = if options.include_rich_header {
            parse_rich_header(&file)
        } else {
            None
        };
        
        let exports = if options.include_exports {
            parse_exports(&file)
        } else {
            None
        };
        
        let imports = if options.include_imports {
            parse_imports(&file)
        } else {
            None
        };
        
        let exception = if options.include_exception {
            parse_exception_directory(&file)
        } else {
            None
        };
        
        let relocs = if options.include_relocations {
            parse_relocs(&file)
        } else {
            None
        };
        
        Ok(Binary {
            file_header: file_header_struct,
            optional_header: optional_header_struct,
            data_directory,
            sections,
            rich_header,
            exception,
            exports,
            imports,
            relocs,
        })
    }
}