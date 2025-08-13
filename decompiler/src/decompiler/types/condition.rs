use capstone::arch::x86::X86Insn;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConditionCode {
    // signed / unsigned distinction matters
    Overflow,             // JO
    NotOverflow,          // JNO
    Below,                // JB / JNAE (unsigned <)
    AboveOrEqual,         // JAE / JNB (unsigned >=)
    Equal,                // JE / JZ
    NotEqual,             // JNE / JNZ
    BelowOrEqual,         // JBE (unsigned <=)
    Above,                // JA  (unsigned >)
    Sign,                 // JS
    NotSign,              // JNS
    ParityEven,           // JP / JPE
    ParityOdd,            // JNP / JPO
    Less,                 // JL / JNGE (signed <)
    GreaterOrEqual,       // JGE / JNL (signed >=)
    LessOrEqual,          // JLE / JNG (signed <=)
    Greater,              // JG / JNLE (signed >)
    CXZ,                  // JCXZ / JECXZ / JRCXZ
}

impl From<X86Insn> for ConditionCode {

    fn from(value: X86Insn) -> Self {
        match value {
            X86Insn::X86_INS_JO => ConditionCode::Overflow,
            X86Insn::X86_INS_JNO => ConditionCode::NotOverflow,
            X86Insn::X86_INS_JB => ConditionCode::Below,
            X86Insn::X86_INS_JAE => ConditionCode::AboveOrEqual,
            X86Insn::X86_INS_JE => ConditionCode::Equal,
            X86Insn::X86_INS_JNE => ConditionCode::NotEqual,
            X86Insn::X86_INS_JBE => ConditionCode::BelowOrEqual,
            X86Insn::X86_INS_JA => ConditionCode::Above,
            X86Insn::X86_INS_JS => ConditionCode::Sign,
            X86Insn::X86_INS_JNS => ConditionCode::NotSign,
            X86Insn::X86_INS_JP => ConditionCode::ParityEven,
            X86Insn::X86_INS_JNP => ConditionCode::ParityOdd,
            X86Insn::X86_INS_JL => ConditionCode::Less,
            X86Insn::X86_INS_JGE => ConditionCode::GreaterOrEqual,
            X86Insn::X86_INS_JLE => ConditionCode::LessOrEqual,
            X86Insn::X86_INS_JG => ConditionCode::Greater,
            X86Insn::X86_INS_JCXZ |
            X86Insn::X86_INS_JECXZ |
            X86Insn::X86_INS_JRCXZ => ConditionCode::CXZ,
            _ => panic!("Invalid condition code"),
        }
    }
}