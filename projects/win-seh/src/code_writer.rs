
pub trait CodeWriter {
    fn write(&self) -> &[u8];
}

#[derive(Debug, Clone, Copy, Default)]
pub struct FaultingCode {
    pub access_violation: bool,
    pub divide_by_zero: bool,
    pub int3: bool,
    pub ud2: bool,
    pub privileged: bool,
}

impl FaultingCode {
    pub fn new() -> Self {
        Self {
            access_violation: false,
            divide_by_zero: false,
            int3: false,
            ud2: false,
            privileged: false
        }
    }

    /// Configures the shellcode to trigger a `STATUS_ACCESS_VIOLATION` (0xC0000005).
    ///
    /// The generated code attempts to write to address 0x2A (42), which is
    /// invalid in most contexts, causing an access violation exception.
    ///
    /// # Assembly
    ///
    /// ```asm
    /// mov eax, 0x2A        ; B8 2A 00 00 00
    /// mov byte [rax], 0    ; C6 00 00
    /// ret                  ; C3
    /// ```
    ///
    /// # Returns
    ///
    /// If the handler skips the faulting instruction (3 bytes), execution
    /// continues at `ret`, returning the value in RAX (42 / 0x2A).
    ///
    /// # Example
    ///
    /// ```
    /// let code = FaultingCode::new().access_violation();
    /// ```
    pub fn access_violation(mut self) -> Self {
        self.access_violation = true;
        self
    }

    /// Configures the shellcode to trigger a `STATUS_INTEGER_DIVIDE_BY_ZERO` (0xC0000094).
    ///
    /// The generated code performs `0 / 0`, which causes a divide-by-zero exception.
    ///
    /// # Assembly
    ///
    /// ```asm
    /// xor rax, rax        ; 48 31 C0
    /// xor rcx, rcx        ; 48 31 C9
    /// div rcx             ; 48 F7 F1
    /// ret                 ; C3
    /// ```
    ///
    /// # Returns
    ///
    /// If the handler skips the faulting instruction (3 bytes), execution
    /// continues at `ret`, returning whatever value is in RAX.
    ///
    /// # Example
    ///
    /// ```
    /// let code = FaultingCode::new().divide_by_zero();
    /// ```
    pub fn divide_by_zero(mut self) -> Self {
        self.divide_by_zero = true;
        self
    }

    /// Configures the shellcode to trigger a `STATUS_BREAKPOINT` (0x80000003).
    ///
    /// The generated code executes the `int3` instruction, which causes a
    /// breakpoint exception.
    ///
    /// # Assembly
    ///
    /// ```asm
    /// int3                 ; CC
    /// ret                  ; C3
    /// ```
    ///
    /// # Returns
    ///
    /// If the handler skips the faulting instruction (1 byte), execution
    /// continues at `ret`, returning whatever value is in RAX.
    ///
    /// # Example
    ///
    /// ```
    /// let code = FaultingCode::new().int3();
    /// ```
    pub fn int3(mut self) -> Self {
        self.int3 = true;
        self
    }

    pub fn ud2(mut self) -> Self {
        self.ud2 = true;
        self
    }

     // lgdt [rip+0] - privileged instruction
    pub fn privileged_instruction(mut self) -> Self {
        self.privileged = true;
        self
    }
}

impl CodeWriter for FaultingCode {
    fn write(&self) -> &[u8] {
        if self.access_violation {
            &[0xb8, 0x2a, 0x00, 0x00, 0x00, 0xc6, 0x00, 0x00, 0xc3]
        } else if self.divide_by_zero {
            &[0x48, 0x31, 0xc0, 0x48, 0x31, 0xc9, 0x48, 0xf7, 0xf1, 0xc3]
        } else if self.int3 {
            &[0xcc, 0xc3]
        } else if self.ud2 {
            &[0x0f, 0x0b, 0xc3]
        } else if self.privileged {
            return &[0x0f, 0x01, 0x15, 0x00, 0x00, 0x00, 0x00, 0xc3];
        } else {
            &[0xc3]
        }
    }
}