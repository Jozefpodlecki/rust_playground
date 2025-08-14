use crate::emulator::RegisterFlags;

pub fn shl(rflags: &mut RegisterFlags, value: u64, count: u8) -> u64 {
    if count == 0 { return value; }

    let result = value << count;

    // Carry Flag = last bit shifted out
    if count <= 64 {
        let carry_bit = (value >> (64 - count)) & 1;
        // if carry_bit == 1 { *rflags |= 1 << 0; } else { *rflags &= !(1 << 0); }
        rflags.update_bit(0, carry_bit == 1); // CF
    }

    // Overflow Flag only if count == 1
    if count == 1 {
        let msb_before = (value >> 63) & 1;
        // *rflags = (*rflags & !(1 << 11)) | (msb_before << 11);
        rflags.update_bit(11, msb_before == 1); // OF
    } else {
        // *rflags &= !(1 << 11);
        rflags.clear_bit(11);
    }

    rflags.update_zf_sf(result);
    rflags.update_parity_flag((result & 0xFF) as u8);

    result
}

pub fn shr(rflags: &mut RegisterFlags, value: u64, count: u8) -> u64 {
    if count == 0 { return value; }

    let result = value >> count;

    if count <= 64 {
        let carry_bit = (value >> (count - 1)) & 1;
        // if carry_bit == 1 { *rflags |= 1 << 0; } else { *rflags &= !(1 << 0); }
        rflags.update_bit(0, carry_bit == 1); // CF
    }

    if count == 1 {
        let msb_before = (value >> 63) & 1;
        // *rflags = (*rflags & !(1 << 11)) | (msb_before << 11);
        rflags.update_bit(11, msb_before == 1); // OF
    } else {
        // *rflags &= !(1 << 11);
        rflags.clear_bit(11);
    }

    rflags.update_zf_sf(result);
    rflags.update_parity_flag((result & 0xFF) as u8);

    result
}
