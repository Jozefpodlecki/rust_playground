#[derive(Debug, Default, Clone)]
pub struct RegisterFlags(u64);

impl From<u64> for RegisterFlags {
    #[inline(always)]
    fn from(value: u64) -> Self {
        Self(value)
    }
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum RegisterFlag {
    CF = 0,
    PF = 2,
    AF = 4,
    ZF = 6,
    SF = 7,
    OF = 11,
}

impl RegisterFlags {
    pub const CF: u64 = 0;
    pub const PF: u64 = 2;
    pub const AF: u64 = 4;
    pub const ZF: u64 = 6;
    pub const SF: u64 = 7;
    pub const OF: u64 = 11;

    pub fn new() -> Self { Self(0) }

    pub fn get(&self, flag: RegisterFlag) -> bool {
        let bit = match flag {
            RegisterFlag::CF => 0,
            RegisterFlag::PF => 2,
            RegisterFlag::AF => 4,
            RegisterFlag::ZF => 6,
            RegisterFlag::SF => 7,
            RegisterFlag::OF => 11,
        };
        (self.0 & (1 << bit)) != 0
    }

    #[inline(always)]
    pub fn set_flag(&mut self, bit: u64, value: bool) {
        // branchless set/clear
        self.0 ^= ((!value as u64).wrapping_add(1) ^ self.0) & (1 << bit);
    }

    #[inline(always)]
    pub fn clear_bit(&mut self, bit: u64) {
        self.0 &= !(1 << bit);
    }

    #[inline(always)]
    pub fn update_bit(&mut self, bit: u64, value: bool) {
        self.0 ^= ((!value as u64).wrapping_add(1) ^ self.0) & (1 << bit);
    }

    #[inline(always)]
    pub fn update_zf_sf(&mut self, result: u64) {
        // Zero Flag (ZF) = bit 6
        self.update_bit(6, result == 0);
        // Sign Flag (SF) = bit 7 (most significant bit)
        self.update_bit(7, (result >> 63) & 1 == 1);
    }

    #[inline(always)]
    fn parity_flag(byte: u8) -> bool {
        // branchless parity calculation
        let mut x = byte;
        x ^= x >> 4;
        x ^= x >> 2;
        x ^= x >> 1;
        (x & 1) == 0
    }

    #[inline(always)]
    pub fn update_parity_flag(&mut self, byte: u8) {
        let parity_even = byte.count_ones() % 2 == 0;
        self.update_bit(2, parity_even); // PF = bit 2
    }

    #[inline(always)]
    pub fn update_add(&mut self, op1: u64, op2: u64, result: u64) {
        let tmp = result;
        let carry = (result < op1) as bool;
        let op1_sign = (op1 >> 63) & 1;
        let op2_sign = (op2 >> 63) & 1;
        let res_sign = (result >> 63) & 1;
        let overflow = (op1_sign == op2_sign) & (op1_sign != res_sign);

        self.0 &= !( (1 << Self::CF) | (1 << Self::PF) | (1 << Self::AF) | (1 << Self::ZF) | (1 << Self::SF) | (1 << Self::OF) );
        self.0 |= (carry as u64) << Self::CF;
        self.0 |= (Self::parity_flag(tmp as u8) as u64) << Self::PF;
        self.0 |= (((op1 ^ op2 ^ tmp) & 0x10 != 0) as u64) << Self::AF;
        self.0 |= ((tmp == 0) as u64) << Self::ZF;
        self.0 |= (((tmp >> 63) & 1) << Self::SF);
        self.0 |= (overflow as u64) << Self::OF;
    }

    #[inline(always)]
    pub fn update_sub(&mut self, op1: u64, op2: u64, result: u64) {
        let tmp = result;
        let borrow = (op1 < op2) as bool;
        let op1_sign = (op1 >> 63) & 1;
        let op2_sign = (op2 >> 63) & 1;
        let res_sign = (result >> 63) & 1;
        let overflow = (op1_sign != op2_sign) & (op1_sign != res_sign);

        self.0 &= !( (1 << Self::CF) | (1 << Self::PF) | (1 << Self::AF) | (1 << Self::ZF) | (1 << Self::SF) | (1 << Self::OF) );
        self.0 |= (borrow as u64) << Self::CF;
        self.0 |= (Self::parity_flag(tmp as u8) as u64) << Self::PF;
        self.0 |= (((op1 ^ op2 ^ tmp) & 0x10 != 0) as u64) << Self::AF;
        self.0 |= ((tmp == 0) as u64) << Self::ZF;
        self.0 |= (((tmp >> 63) & 1) << Self::SF);
        self.0 |= (overflow as u64) << Self::OF;
    }

    #[inline(always)]
    pub fn update_logic(&mut self, result: u64) {
        // CF and OF are cleared automatically
        self.0 &= !( (1 << Self::CF) | (1 << Self::OF) | (1 << Self::PF) | (1 << Self::ZF) | (1 << Self::SF) );
        self.0 |= (Self::parity_flag(result as u8) as u64) << Self::PF;
        self.0 |= ((result == 0) as u64) << Self::ZF;
        self.0 |= ((result >> 63) & 1) << Self::SF;
    }

    pub fn update_adc(&mut self, op1: u64, op2: u64, carry_in: bool, result: u64) {
        let tmp = result;
        let carry_out = (op1 as u128) + (op2 as u128) + (carry_in as u128) > u64::MAX as u128;
        let op1_sign = (op1 >> 63) & 1;
        let op2_sign = (op2 >> 63) & 1;
        let res_sign = (result >> 63) & 1;
        let overflow = ((op1_sign == op2_sign) & (op1_sign != res_sign)) != false;

        self.0 &= !((1 << Self::CF) | (1 << Self::PF) | (1 << Self::AF ) | (1 << Self::ZF) | (1 << Self::SF) | (1 << Self::OF));
        self.0 |= (carry_out as u64) << Self::CF;
        self.0 |= (Self::parity_flag(tmp as u8) as u64) << Self::PF;
        self.0 |= (((op1 ^ op2 ^ tmp) & 0x10 != 0) as u64) << Self::AF;
        self.0 |= ((tmp == 0) as u64) << Self::ZF;
        self.0 |= (((tmp >> 63) & 1) << Self::SF);
        self.0 |= (overflow as u64) << Self::OF;
    }


    #[inline(always)]
    pub fn update_sub_no_cf(&mut self, lhs: u64, rhs: u64, result: u64) {
        // Zero Flag
        self.update_bit(Self::ZF, result == 0);

        // Sign Flag
        self.update_bit(Self::SF, (result >> 63) & 1 == 1);

        // Overflow Flag
        let lhs_s = lhs as i64;
        let rhs_s = rhs as i64;
        let res_s = result as i64;
        let overflow = (lhs_s < 0 && rhs_s > 0 && res_s >= 0) ||
                       (lhs_s >= 0 && rhs_s < 0 && res_s < 0);
        self.update_bit(Self::OF, overflow);

        // Parity Flag
        self.update_parity_flag(result as u8);
    }

    #[inline(always)]
    pub fn raw(&self) -> u64 { self.0 }
}

// pub fn update_parity_flag(rflags: &mut u64, byte: u8) {
//     let count = byte.count_ones();
//     if count % 2 == 0 {
//         *rflags |= 1 << 2; // PF
//     } else {
//         *rflags &= !(1 << 2);
//     }
// }

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

// pub fn update_zf_sf(rflags: &mut u64, result: u64) {
//     if result == 0 { *rflags |= 1 << 6; } else { *rflags &= !(1 << 6); }
//     if (result >> 63) & 1 == 1 { *rflags |= 1 << 7; } else { *rflags &= !(1 << 7); }
// }

// pub fn update_flags_add(rflags: &mut u64, op1: u64, op2: u64, result: u64) {

//     // Zero Flag (ZF): Set if result == 0
//     if result == 0 {
//         *rflags |= 1 << 6;
//     } else {
//         *rflags &= !(1 << 6);
//     }

//     // Sign Flag (SF): Set if most significant bit is set (for 64-bit, bit 63)
//     if (result >> 63) & 1 == 1 {
//         *rflags |= 1 << 7;
//     } else {
//         *rflags &= !(1 << 7);
//     }

//     // Carry Flag (CF): Set if unsigned overflow occurs
//     if result < op1 {
//         *rflags |= 1 << 0;
//     } else {
//         *rflags &= !(1 << 0);
//     }

//     // Overflow Flag (OF): Set if signed overflow occurs
//     let op1_sign = (op1 >> 63) & 1;
//     let op2_sign = (op2 >> 63) & 1;
//     let res_sign = (result >> 63) & 1;
//     if op1_sign == op2_sign && op1_sign != res_sign {
//         *rflags |= 1 << 11;
//     } else {
//         *rflags &= !(1 << 11);
//     }

//     // You can add Parity Flag (PF) and others similarly
// }

// pub fn update_flags_sub_no_cf(rflags: &mut u64, lhs: u64, rhs: u64, result: u64) {
//     // CF is not modified
//     // ZF
//     if result == 0 {
//         *rflags |= 1 << 6;
//     } else {
//         *rflags &= !(1 << 6);
//     }
//     // SF
//     if (result >> 63) & 1 == 1 {
//         *rflags |= 1 << 7;
//     } else {
//         *rflags &= !(1 << 7);
//     }
//     // OF
//     let lhs_s = lhs as i64;
//     let rhs_s = rhs as i64;
//     let res_s = result as i64;
//     if (lhs_s < 0 && rhs_s > 0 && res_s >= 0) ||
//         (lhs_s >= 0 && rhs_s < 0 && res_s < 0) {
//         *rflags |= 1 << 11;
//     } else {
//         *rflags &= !(1 << 11);
//     }
//     // PF
//     if (result & 0xFF).count_ones() % 2 == 0 {
//         *rflags |= 1 << 2;
//     } else {
//         *rflags &= !(1 << 2);
//     }
// }

// pub fn update_flags_sub(rflags: &mut u64, op1: u64, op2: u64, result: u64) {

//     // Zero Flag (ZF)
//     if result == 0 {
//         *rflags |= 1 << 6;
//     } else {
//         *rflags &= !(1 << 6);
//     }

//     // Sign Flag (SF)
//     if (result >> 63) & 1 == 1 {
//         *rflags |= 1 << 7;
//     } else {
//         *rflags &= !(1 << 7);
//     }

//     // Carry Flag (CF): Set if borrow (if op1 < op2)
//     if op1 < op2 {
//         *rflags |= 1 << 0;
//     } else {
//         *rflags &= !(1 << 0);
//     }

//     // Overflow Flag (OF): Signed overflow detection for subtraction
//     let op1_sign = (op1 >> 63) & 1;
//     let op2_sign = (op2 >> 63) & 1;
//     let res_sign = (result >> 63) & 1;
//     if op1_sign != op2_sign && op1_sign != res_sign {
//         *rflags |= 1 << 11;
//     } else {
//         *rflags &= !(1 << 11);
//     }
// }

// pub fn update_flags_adc(rflags: &mut u64, val1: u64, val2: u64, carry: u64, result: u64) {
//     // Detect carry (CF) for 64-bit unsigned add with carry
//     let carry_out = (val1 as u128) + (val2 as u128) + (carry as u128) > u64::MAX as u128;
//     if carry_out {
//         *rflags |= 1 << 0;
//     } else {
//         *rflags &= !(1 << 0);
//     }

//     // Overflow flag (OF)
//     let sign_bit = 1u64 << 63;
//     let of = ((val1 ^ result) & (val2 ^ result) & sign_bit) != 0;
//     if of {
//         *rflags |= 1 << 11;
//     } else {
//         *rflags &= !(1 << 11);
//     }

//     // Zero Flag (ZF)
//     if result == 0 {
//         *rflags |= 1 << 6;
//     } else {
//         *rflags &= !(1 << 6);
//     }

//     // Sign Flag (SF)
//     if (result & sign_bit) != 0 {
//         *rflags |= 1 << 7;
//     } else {
//         *rflags &= !(1 << 7);
//     }

//     // Adjust Flag (AF) - bit 4
//     let af = ((val1 ^ val2 ^ result) & 0x10) != 0;
//     if af {
//         *rflags |= 1 << 4;
//     } else {
//         *rflags &= !(1 << 4);
//     }

//     // Parity Flag (PF) - bit 2
//     let parity = (result as u8).count_ones() % 2 == 0;
//     if parity {
//         *rflags |= 1 << 2;
//     } else {
//         *rflags &= !(1 << 2);
//     }
// }

// pub fn update_flags_logical(rflags: &mut u64, result: u64) {
//     // Clear CF and OF flags (bits 0 and 11)
//     *rflags &= !((1 << 0) | (1 << 11));

//     // Set or clear Zero Flag (ZF, bit 6)
//     if result == 0 {
//         *rflags |= 1 << 6;
//     } else {
//         *rflags &= !(1 << 6);
//     }

//     // Set or clear Sign Flag (SF, bit 7)
//     if (result >> 63) & 1 == 1 {
//         *rflags |= 1 << 7;
//     } else {
//         *rflags &= !(1 << 7);
//     }
// }

// pub fn update_flags_after_logic(rflags: &mut u64, result: u64) {
//     // Clear CF and OF
//     *rflags &= !((1 << 0) | (1 << 11));

//     // Set ZF
//     if result == 0 {
//         *rflags |= 1 << 6;
//     } else {
//         *rflags &= !(1 << 6);
//     }

//     // Set SF (sign flag, highest bit for 64-bit)
//     if (result >> 63) & 1 == 1 {
//         *rflags |= 1 << 7;
//     } else {
//         *rflags &= !(1 << 7);
//     }

//     // Set PF (parity flag: 1 if number of 1 bits in least-significant byte is even)
//     if (result as u8).count_ones() % 2 == 0 {
//         *rflags |= 1 << 2;
//     } else {
//         *rflags &= !(1 << 2);
//     }
// }
