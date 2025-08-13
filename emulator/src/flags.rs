pub struct RegisterFlags(u64);

impl RegisterFlags {
    
}

pub fn update_parity_flag(rflags: &mut u64, byte: u8) {
    let count = byte.count_ones();
    if count % 2 == 0 {
        *rflags |= 1 << 2; // PF
    } else {
        *rflags &= !(1 << 2);
    }
}

// fn update_parity_flag(&mut self, byte: u8) {
//     // count number of set bits in byte
//     let mut count = 0;
//     let mut val = byte;
//     for _ in 0..8 {
//         count += val & 1;
//         val >>= 1;
//     }
//     if count % 2 == 0 {
//         *rflags |= 1 << 2;  // PF
//     } else {
//         *rflags &= !(1 << 2);
//     }
// }

pub fn update_zf_sf(rflags: &mut u64, result: u64) {
    if result == 0 { *rflags |= 1 << 6; } else { *rflags &= !(1 << 6); }
    if (result >> 63) & 1 == 1 { *rflags |= 1 << 7; } else { *rflags &= !(1 << 7); }
}

pub fn update_flags_add(rflags: &mut u64, op1: u64, op2: u64, result: u64) {

    // Zero Flag (ZF): Set if result == 0
    if result == 0 {
        *rflags |= 1 << 6;
    } else {
        *rflags &= !(1 << 6);
    }

    // Sign Flag (SF): Set if most significant bit is set (for 64-bit, bit 63)
    if (result >> 63) & 1 == 1 {
        *rflags |= 1 << 7;
    } else {
        *rflags &= !(1 << 7);
    }

    // Carry Flag (CF): Set if unsigned overflow occurs
    if result < op1 {
        *rflags |= 1 << 0;
    } else {
        *rflags &= !(1 << 0);
    }

    // Overflow Flag (OF): Set if signed overflow occurs
    let op1_sign = (op1 >> 63) & 1;
    let op2_sign = (op2 >> 63) & 1;
    let res_sign = (result >> 63) & 1;
    if op1_sign == op2_sign && op1_sign != res_sign {
        *rflags |= 1 << 11;
    } else {
        *rflags &= !(1 << 11);
    }

    // You can add Parity Flag (PF) and others similarly
}

pub fn update_flags_sub_no_cf(rflags: &mut u64, lhs: u64, rhs: u64, result: u64) {
    // CF is not modified
    // ZF
    if result == 0 {
        *rflags |= 1 << 6;
    } else {
        *rflags &= !(1 << 6);
    }
    // SF
    if (result >> 63) & 1 == 1 {
        *rflags |= 1 << 7;
    } else {
        *rflags &= !(1 << 7);
    }
    // OF
    let lhs_s = lhs as i64;
    let rhs_s = rhs as i64;
    let res_s = result as i64;
    if (lhs_s < 0 && rhs_s > 0 && res_s >= 0) ||
        (lhs_s >= 0 && rhs_s < 0 && res_s < 0) {
        *rflags |= 1 << 11;
    } else {
        *rflags &= !(1 << 11);
    }
    // PF
    if (result & 0xFF).count_ones() % 2 == 0 {
        *rflags |= 1 << 2;
    } else {
        *rflags &= !(1 << 2);
    }
}

pub fn update_flags_sub(rflags: &mut u64, op1: u64, op2: u64, result: u64) {

    // Zero Flag (ZF)
    if result == 0 {
        *rflags |= 1 << 6;
    } else {
        *rflags &= !(1 << 6);
    }

    // Sign Flag (SF)
    if (result >> 63) & 1 == 1 {
        *rflags |= 1 << 7;
    } else {
        *rflags &= !(1 << 7);
    }

    // Carry Flag (CF): Set if borrow (if op1 < op2)
    if op1 < op2 {
        *rflags |= 1 << 0;
    } else {
        *rflags &= !(1 << 0);
    }

    // Overflow Flag (OF): Signed overflow detection for subtraction
    let op1_sign = (op1 >> 63) & 1;
    let op2_sign = (op2 >> 63) & 1;
    let res_sign = (result >> 63) & 1;
    if op1_sign != op2_sign && op1_sign != res_sign {
        *rflags |= 1 << 11;
    } else {
        *rflags &= !(1 << 11);
    }
}

pub fn update_flags_adc(rflags: &mut u64, val1: u64, val2: u64, carry: u64, result: u64) {
    // Detect carry (CF) for 64-bit unsigned add with carry
    let carry_out = (val1 as u128) + (val2 as u128) + (carry as u128) > u64::MAX as u128;
    if carry_out {
        *rflags |= 1 << 0;
    } else {
        *rflags &= !(1 << 0);
    }

    // Overflow flag (OF)
    let sign_bit = 1u64 << 63;
    let of = ((val1 ^ result) & (val2 ^ result) & sign_bit) != 0;
    if of {
        *rflags |= 1 << 11;
    } else {
        *rflags &= !(1 << 11);
    }

    // Zero Flag (ZF)
    if result == 0 {
        *rflags |= 1 << 6;
    } else {
        *rflags &= !(1 << 6);
    }

    // Sign Flag (SF)
    if (result & sign_bit) != 0 {
        *rflags |= 1 << 7;
    } else {
        *rflags &= !(1 << 7);
    }

    // Adjust Flag (AF) - bit 4
    let af = ((val1 ^ val2 ^ result) & 0x10) != 0;
    if af {
        *rflags |= 1 << 4;
    } else {
        *rflags &= !(1 << 4);
    }

    // Parity Flag (PF) - bit 2
    let parity = (result as u8).count_ones() % 2 == 0;
    if parity {
        *rflags |= 1 << 2;
    } else {
        *rflags &= !(1 << 2);
    }
}

pub fn update_flags_logical(rflags: &mut u64, result: u64) {
    // Clear CF and OF flags (bits 0 and 11)
    *rflags &= !((1 << 0) | (1 << 11));

    // Set or clear Zero Flag (ZF, bit 6)
    if result == 0 {
        *rflags |= 1 << 6;
    } else {
        *rflags &= !(1 << 6);
    }

    // Set or clear Sign Flag (SF, bit 7)
    if (result >> 63) & 1 == 1 {
        *rflags |= 1 << 7;
    } else {
        *rflags &= !(1 << 7);
    }
}

pub fn update_flags_after_logic(rflags: &mut u64, result: u64) {
    // Clear CF and OF
    *rflags &= !((1 << 0) | (1 << 11));

    // Set ZF
    if result == 0 {
        *rflags |= 1 << 6;
    } else {
        *rflags &= !(1 << 6);
    }

    // Set SF (sign flag, highest bit for 64-bit)
    if (result >> 63) & 1 == 1 {
        *rflags |= 1 << 7;
    } else {
        *rflags &= !(1 << 7);
    }

    // Set PF (parity flag: 1 if number of 1 bits in least-significant byte is even)
    if (result as u8).count_ones() % 2 == 0 {
        *rflags |= 1 << 2;
    } else {
        *rflags &= !(1 << 2);
    }
}
