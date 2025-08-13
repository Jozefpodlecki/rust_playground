use crate::flags::{self, update_parity_flag, update_zf_sf};

pub fn shl(rflags: &mut u64, value: u64, count: u8) -> u64 {
    if count == 0 { return value; }

    let result = value << count;

    // Carry Flag = last bit shifted out
    if count <= 64 {
        let carry_bit = (value >> (64 - count)) & 1;
        if carry_bit == 1 { *rflags |= 1 << 0; } else { *rflags &= !(1 << 0); }
    }

    // Overflow Flag only if count == 1
    if count == 1 {
        let msb_before = (value >> 63) & 1;
        *rflags = (*rflags & !(1 << 11)) | (msb_before << 11);
    } else {
        *rflags &= !(1 << 11);
    }

    update_zf_sf(rflags, result);
    update_parity_flag(rflags, (result & 0xFF) as u8);

    result
}

pub fn shr(rflags: &mut u64, value: u64, count: u8) -> u64 {
    if count == 0 { return value; }

    let result = value >> count;

    if count <= 64 {
        let carry_bit = (value >> (count - 1)) & 1;
        if carry_bit == 1 { *rflags |= 1 << 0; } else { *rflags &= !(1 << 0); }
    }

    if count == 1 {
        let msb_before = (value >> 63) & 1;
        *rflags = (*rflags & !(1 << 11)) | (msb_before << 11);
    } else {
        *rflags &= !(1 << 11);
    }

    update_zf_sf(rflags, result);
    update_parity_flag(rflags, (result & 0xFF) as u8);

    result
}
