use utils::{print, println};

use crate::types::*;

pub fn dump_all(info: &FullCpuidInfo) {
    dump_basic_info(info);
    dump_processor_info(info);
    dump_features(info);
    dump_brand(info);
    dump_cache(info);
    dump_topology(info);
    dump_address(info);
    dump_power(info);
    dump_frequency(info);
    dump_tsc(info);
    dump_extended(info);
}

fn dump_basic_info(info: &FullCpuidInfo) {
    println!("=== BASIC CPU INFO ===");
    println!("");
    println!("Vendor: {}", info.basic.vendor_id.as_str());
    println!("Max Leaf: 0x{:X}", info.basic.max_leaf);
    if let Ok(s) = core::str::from_utf8(&info.basic.vendor_string) {
        println!("Vendor String: {}", s);
    }
    println!("");
}

fn dump_processor_info(info: &FullCpuidInfo) {
    println!("=== PROCESSOR INFO ===");
    println!("");
    println!("Family: {}", info.processor.family);
    println!("Model: {}", info.processor.model);
    println!("Stepping: {}", info.processor.stepping);
    println!("Type: {}", info.processor.processor_type.as_str());
    println!("Extended Family: {}", info.processor.extended_family);
    println!("Extended Model: {}", info.processor.extended_model);
    println!("Brand Index: {}", info.processor.brand_index);
    println!("");
}

fn dump_features(info: &FullCpuidInfo) {
    println!("=== CPU FEATURES ===");
    println!("");
    
    println!("Standard EDX: 0x{:08X}", info.features.standard_edx);
    println!("  {}", StandardFeatures(info.features.standard_edx));
    
    println!("");
    println!("Extended ECX: 0x{:08X}", info.features.extended_ecx);
    println!("  {}", ExtendedFeatures(info.features.extended_ecx));
    
    println!("");
    println!("Extended 7 EBX: 0x{:08X}", info.features.extended_7_ebx);
    println!("  {}", Extended7EbFeatures(info.features.extended_7_ebx));
    
    println!("");
    println!("Extended 7 ECX: 0x{:08X}", info.features.extended_7_ecx);
    println!("  {}", Extended7EcFeatures(info.features.extended_7_ecx));
    
    println!("");
    println!("Extended 7 EDX: 0x{:08X}", info.features.extended_7_edx);
    println!("  {}", Extended7EdFeatures(info.features.extended_7_edx));
    
    println!("");
    println!("Extended 7.1 EAX: 0x{:08X}", info.features.extended_7_1_eax);
    println!("Extended 7.1 EBX: 0x{:08X}", info.features.extended_7_1_ebx);
    println!("Extended 7.1 ECX: 0x{:08X}", info.features.extended_7_1_ecx);
    println!("Extended 7.1 EDX: 0x{:08X}", info.features.extended_7_1_edx);
    
    println!("");
}

fn dump_brand(info: &FullCpuidInfo) {
    println!("=== BRAND STRING ===");
    println!("");
    if let Ok(s) = info.brand.as_str() {
        println!("{}", s);
    }
    println!("");
}

fn dump_cache(info: &FullCpuidInfo) {
    println!("=== CACHE INFO ===");
    println!("");
    
    let mut found = false;
    for (i, cache_opt) in info.cache.iter().enumerate() {
        if let Some(cache) = cache_opt {
            found = true;
            let cache_type_str = match cache.cache_type {
                1 => "Data",
                2 => "Instruction",
                3 => "Unified",
                _ => "Unknown",
            };
            
            // Manually format size with KB/MB using write! or print! statements
            if cache.cache_size_kb >= 1024 {
                let mb = cache.cache_size_kb as f32 / 1024.0;
                // Print with one decimal place
                let whole = mb as u32;
                let frac = ((mb - whole as f32) * 10.0) as u32;
                print!("L{} {}: {}.{} MB, ", cache.cache_level, cache_type_str, whole, frac);
            } else {
                print!("L{} {}: {} KB, ", cache.cache_level, cache_type_str, cache.cache_size_kb);
            }
            
            println!("Ways: {}, Line: {} bytes", 
                cache.ways_of_associativity,
                cache.line_size_bytes
            );
        }
    }
    
    if !found {
        println!("(none)");
    }
    println!("");
}

fn dump_topology(info: &FullCpuidInfo) {
    println!("=== TOPOLOGY ===");
    println!("");
    
    if let Some(top) = &info.topology {
        println!("SMT Mask: 0x{:X}", top.smt_mask);
        println!("Core Mask: 0x{:X}", top.core_mask);
        println!("SMT Shift: {}", top.smt_shift);
        println!("Core Shift: {}", top.core_shift);
        println!("");
        
        let smt_count = top.smt_mask + 1;
        let core_count = top.core_mask + 1;
        let total = smt_count * core_count;
        
        println!("Threads per Core: {}", smt_count);
        println!("Cores: {}", core_count);
        println!("Logical CPUs: {}", total);
    } else {
        println!("(not supported)");
    }
    println!("");
}

fn dump_address(info: &FullCpuidInfo) {
    println!("=== ADDRESS SPACE ===");
    println!("");
    
    if let Some(addr) = &info.address {
        println!("Physical Address Bits: {}", addr.physical_addr_bits);
        println!("Virtual Address Bits: {}", addr.virtual_addr_bits);
        println!("");
        
        let max_ram = 1u64 << addr.physical_addr_bits;
        
        // Manually format RAM size without format!
        if max_ram >= 1_000_000_000_000 {
            let tb = max_ram / 1_000_000_000_000;
            println!("Max RAM: {} TB", tb);
        } else if max_ram >= 1_000_000_000 {
            let gb = max_ram / 1_000_000_000;
            println!("Max RAM: {} GB", gb);
        } else if max_ram >= 1_000_000 {
            let mb = max_ram / 1_000_000;
            println!("Max RAM: {} MB", mb);
        } else if max_ram >= 1_000 {
            let kb = max_ram / 1_000;
            println!("Max RAM: {} KB", kb);
        } else {
            println!("Max RAM: {} bytes", max_ram);
        }
    } else {
        println!("(not supported)");
    }
    println!("");
}

fn dump_power(info: &FullCpuidInfo) {
    println!("=== POWER MANAGEMENT ===");
    println!("");
    
    if let Some(power) = &info.power {
        println!("Digital Thermal Sensor: {}", power.digital_thermal_sensor);
        println!("Intel Thermal Monitor: {}", power.intel_thermal_monitor);
        println!("Performance Bias: {}", power.performance_bias);
        println!("Hardware Coordination: {}", power.hardware_coordination);
        println!("Energy Performance Preference: {}", power.energy_performance_preference);
        println!("Turbo Boost Max 3.0: {}", power.turbo_boost_max_3);
        println!("Hardware P-States: {}", power.hardware_p_states);
        println!("Hardware P-States Guaranteed: {}", power.hardware_p_states_guaranteed);
    } else {
        println!("(not supported)");
    }
    println!("");
}

fn dump_frequency(info: &FullCpuidInfo) {
    println!("=== FREQUENCY INFO ===");
    println!("");
    
    if let Some(freq) = &info.frequency {
        println!("Base Frequency: {} MHz", freq.base_mhz);
        println!("Max Frequency: {} MHz", freq.max_mhz);
        println!("Bus Frequency: {} MHz", freq.bus_mhz);
        if freq.edx != 0 {
            println!("EDX: 0x{:08X}", freq.edx);
        }
    } else {
        println!("(not supported - CPUID leaf 0x16 not available or returned zeros)");
    }
    println!("");
}

fn dump_tsc(info: &FullCpuidInfo) {
    println!("=== TSC INFO ===");
    println!("");
    
    if let Some(tsc) = &info.tsc {
        println!("Denominator: {}", tsc.denominator);
        println!("Numerator: {}", tsc.numerator);
        if tsc.nominal_frequency > 0 {
            println!("Nominal Frequency: {} Hz", tsc.nominal_frequency);
        } else {
            println!("Nominal Frequency: (not available)");
        }
    } else {
        println!("(not supported)");
    }
    println!("");
}

fn dump_extended(info: &FullCpuidInfo) {
    println!("=== EXTENDED CPUID INFO ===");
    println!("");
    
    println!("Max Extended Leaf: 0x{:X}", info.extended.max_extended_leaf);
    println!("");
    
    if let Some(ext) = &info.extended.extended_features {
        println!("Extended Features:");
        println!("  EAX: 0x{:08X}", ext.eax);
        println!("  EBX: 0x{:08X}", ext.ebx);
        println!("  ECX: 0x{:08X}", ext.ecx);
        println!("    {}", Extended80000001EcxFeatures(ext.ecx));
        println!("  EDX: 0x{:08X}", ext.edx);
        println!("    {}", Extended80000001EdxFeatures(ext.edx));
        println!("");
    }
    
    if let Some(l1) = &info.extended.l1_cache_tlb {
        println!("L1 Cache/TLB Info:");
        println!("  EAX: 0x{:08X}", l1.eax);
        println!("  EBX: 0x{:08X}", l1.ebx);
        println!("  ECX: 0x{:08X}", l1.ecx);
        println!("  EDX: 0x{:08X}", l1.edx);
        println!("");
        println!("  L1 Data Cache:");
        println!("    Size: {} KB", l1.l1_data_cache_size);
        println!("    Associativity: {}", l1.l1_data_cache_associativity);
        println!("    Lines per Tag: {}", l1.l1_data_cache_lines_per_tag);
        println!("    Line Size: {} bytes", l1.l1_data_cache_line_size);
        println!("");
        println!("  L1 Instruction Cache:");
        println!("    Size: {} KB", l1.l1_instruction_cache_size);
        println!("    Associativity: {}", l1.l1_instruction_cache_associativity);
        println!("    Lines per Tag: {}", l1.l1_instruction_cache_lines_per_tag);
        println!("    Line Size: {} bytes", l1.l1_instruction_cache_line_size);
        println!("");
    }
    
    if let Some(l2) = &info.extended.l2_cache_tlb {
        println!("L2 Cache/TLB Info:");
        println!("  EAX: 0x{:08X}", l2.eax);
        println!("  EBX: 0x{:08X}", l2.ebx);
        println!("  ECX: 0x{:08X}", l2.ecx);
        println!("  EDX: 0x{:08X}", l2.edx);
        println!("");
        println!("  L2 Cache:");
        println!("    Size: {} KB", l2.l2_cache_size);
        println!("    Associativity: {}", l2.l2_cache_associativity);
        println!("    Line Size: {} bytes", l2.l2_cache_line_size);
        println!("");
        println!("  L2 TLB (2M/4M pages):");
        println!("    Associativity: {}", l2.l2_tlb_2m_4m_associativity);
        println!("    Entries: {}", l2.l2_tlb_2m_4m_entries);
        println!("");
        println!("  L2 TLB (4K pages):");
        println!("    Associativity: {}", l2.l2_tlb_4k_associativity);
        println!("    Entries: {}", l2.l2_tlb_4k_entries);
        println!("");
    }
    
    if let Some(pm) = &info.extended.power_management {
        println!("Extended Power Management:");
        println!("  EAX: 0x{:08X}", pm.eax);
        println!("  EBX: 0x{:08X}", pm.ebx);
        println!("  ECX: 0x{:08X}", pm.ecx);
        println!("  EDX: 0x{:08X}", pm.edx);
        println!("");
        println!("  Features:");
        println!("    Invariant TSC: {}", pm.invariant_tsc);
        println!("    Thermal Monitor: {}", pm.thermal_monitor);
        println!("    Software Thermal Control: {}", pm.software_thermal_control);
        println!("    Hardware Thermal Control: {}", pm.hardware_thermal_control);
        println!("    Performance Status: {}", pm.performance_status);
        println!("    Hardware P-State: {}", pm.hardware_p_state);
        println!("    Software P-State: {}", pm.software_p_state);
        println!("    TSC Scale: {}", pm.tsc_scale);
        println!("");
    }
    
    if let Some(enc) = &info.extended.memory_encryption {
        println!("Memory Encryption:");
        println!("  EAX: 0x{:08X}", enc.eax);
        println!("  EBX: 0x{:08X}", enc.ebx);
        println!("  ECX: 0x{:08X}", enc.ecx);
        println!("  EDX: 0x{:08X}", enc.edx);
        println!("");
    }
}