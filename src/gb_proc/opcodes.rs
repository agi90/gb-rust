macro_rules! op_codes {
    ($($element: ident: ($tostring: expr, $hex: expr, $cycles: expr)),+) => {
        #[derive(Debug, Copy, Clone, PartialEq)]
        pub enum OpCode {
            $($element),+
        }

        impl OpCode {
            pub fn to_byte(&self) -> u8 {
                match self {
                    $(&OpCode::$element => $hex),*,
                }
            }

            pub fn from_byte(hex: u8) -> OpCode {
                match hex {
                    $($hex => OpCode::$element),*,
                    _ => panic!("Op code not implemented")
                }
            }

            pub fn to_string(&self) -> &'static str {
                match self {
                    $(&OpCode::$element => $tostring),*,
                }
            }

            pub fn get_cycles(&self) -> usize {
                match self {
                    $(&OpCode::$element => $cycles),*,
                }
            }
        }
    }
}

op_codes!(
    // LD nn,n
    //
    // Put value nn into n
    //
    // nn = B,C,D,E,H,K,BC,DE,HL,SP
    // n = 8 bit immediate value
    LdBn: ("LD B,", 0x06, 8),
    LdCn: ("LD C,", 0x0E, 8),
    LdDn: ("LD D,", 0x16, 8),
    LdEn: ("LD E,", 0x1E, 8),
    LdHn: ("LD H,", 0x26, 8),
    LdLn: ("LD L,", 0x2E, 8),

    // LD r1,r2
    //
    // Put value r2 into r1
    //
    // r1,r2 = A,B,C,D,E,H,L,(HL)
    LdAA:  ("LD A,A",    0x7F, 4),
    LdAB:  ("LD A,B",    0x78, 4),
    LdAC:  ("LD A,C",    0x79, 4),
    LdAD:  ("LD A,D",    0x7A, 4),
    LdAE:  ("LD A,E",    0x7B, 4),
    LdAH:  ("LD A,H",    0x7C, 4),
    LdAL:  ("LD A,L",    0x7D, 4),
    LdAHL: ("LD A,(HL)", 0x7E, 8),

    LdBB:  ("LD B,B",    0x40, 4),
    LdBC:  ("LD B,C",    0x41, 4),
    LdBD:  ("LD B,D",    0x42, 4),
    LdBE:  ("LD B,E",    0x43, 4),
    LdBH:  ("LD B,H",    0x44, 4),
    LdBL:  ("LD B,L",    0x45, 4),
    LdBHL: ("LD B,(HL)", 0x46, 8),

    LdCB:  ("LD C,B",    0x48, 4),
    LdCC:  ("LD C,C",    0x49, 4),
    LdCD:  ("LD C,D",    0x4A, 4),
    LdCE:  ("LD C,E",    0x4B, 4),
    LdCH:  ("LD C,H",    0x4C, 4),
    LdCL:  ("LD C,L",    0x4D, 4),
    LdCHL: ("LD C,(HL)", 0x4E, 8),

    LdDB:  ("LD D,B",    0x50, 4),
    LdDC:  ("LD D,C",    0x51, 4),
    LdDD:  ("LD D,D",    0x52, 4),
    LdDE:  ("LD D,E",    0x53, 4),
    LdDH:  ("LD D,H",    0x54, 4),
    LdDL:  ("LD D,L",    0x55, 4),
    LdDHL: ("LD D,(HL)", 0x56, 8),

    LdEB:  ("LD E,B",    0x58, 4),
    LdEC:  ("LD E,C",    0x59, 4),
    LdED:  ("LD E,D",    0x5A, 4),
    LdEE:  ("LD E,E",    0x5B, 4),
    LdEH:  ("LD E,H",    0x5C, 4),
    LdEL:  ("LD E,L",    0x5D, 4),
    LdEHL: ("LD E,(HL)", 0x5E, 8),

    LdHB:  ("LD H,B",    0x60, 4),
    LdHC:  ("LD H,C",    0x61, 4),
    LdHD:  ("LD H,D",    0x62, 4),
    LdHE:  ("LD H,E",    0x63, 4),
    LdHH:  ("LD H,H",    0x64, 4),
    LdHL:  ("LD H,L",    0x65, 4),
    LdHHL: ("LD H,(HL)", 0x66, 8),

    LdLB:  ("LD L,B",    0x68, 4),
    LdLC:  ("LD L,C",    0x69, 4),
    LdLD:  ("LD L,D",    0x6A, 4),
    LdLE:  ("LD L,E",    0x6B, 4),
    LdLH:  ("LD L,H",    0x6C, 4),
    LdLL:  ("LD L,L",    0x6D, 4),
    LdLHL: ("LD L,(HL)", 0x6E, 8),

    LdHLB:  ("LD (HL),B", 0x70, 8),
    LdHLC:  ("LD (HL),C", 0x71, 8),
    LdHLD:  ("LD (HL),D", 0x72, 8),
    LdHLE:  ("LD (HL),E", 0x73, 8),
    LdHLH:  ("LD (HL),H", 0x74, 8),
    LdHLL:  ("LD (HL),L", 0x75, 8),
    LdHLHL: ("LD (HL),",  0x36, 12),

    // LD A,n
    //
    // Put value n into A
    //
    // n = A,B,C,D,E,H,L,(BC),(DE),(HL),(nn),#
    // nn = two byte immediate value. (LS byte first.)
    LdABC: ("LD A,(BC)", 0x0A, 8),
    LdADE: ("LD A,(DE)", 0x1A, 8),
    LdAnn: ("LD A,(nn)", 0xFA, 16),
    LdAx:  ("LD A,#",    0x3E, 8),

    // LD n,A
    //
    // Put value A into n
    //
    // n = A,B,C,D,E,H,L,(BC),(DE),(HL),(nn)
    // nn = two byte immediate value. (LS byte first)
    LdBCA: ("LD (BC),A", 0x02, 8),
    LdDEA: ("LD (DE),A", 0x12, 8),
    LdHLA: ("LD (HL),A", 0x77, 8),
    LdnnA: ("LD (nn),A", 0xEA, 16),

    // LD A,(C)
    //
    // Put value at address $FF00 + register C into A
    // Same as: LD A, ($FF00+C)
    LdAFFC: ("LD A,($FF00+C)", 0xF2, 8),

    // LD (C),A
    //
    // Put A into address $FF00 + register C
    LdFFCA: ("LD ($FF00+C),A", 0xE2, 8),

    // LDD A,(HL)
    //
    // Put value at address HL into A. Decrement HL.
    // Same as: LD A,(HL) - DEC HL
    LddAHL: ("LDD A,(HL)", 0x3A, 8),

    // LDD (HL),A
    //
    // Put A into memoty address HL. Decrement HL.
    // Same as: LD (HL),A - DEC HL
    LddHLA: ("LDD (HL),A", 0x32, 8),

    // LDI A,(HL)
    //
    // Put value at address HL into A. Increment HL.
    // Same as: LD A,(HL) - INC HL
    LdiAHL: ("LDI A,(HL)", 0x2A, 8),

    // LDI (HL),A
    //
    // Put A into memory address HL. Increment HL.
    // Same as: LD (HL),A - INC HL
    LdiHLA: ("LDI (HL),A", 0x22, 8),

    // LDH (n),A
    //
    // Put A into memory address $FF00+n
    //
    // n = one byte immediate value
    LdhNA: ("LD ($FF00+n),A", 0xE0, 12),

    // LDH A,(n)
    //
    // Put memory address $FF00+n into A.
    //
    // n = one byte immediate value
    LdhAHL: ("LD A,($FF00+n)", 0xF0, 12),

    // LD n,nn
    //
    // Put value nn into n
    //
    // n = BC,DE,HL,SP
    // nn = 16 bit immediate value
    LdBCnn: ("LD BC,nn", 0x01, 12),
    LdDEnn: ("LD DE,nn", 0x11, 12),
    LdHLnn: ("LD HL,nn", 0x21, 12),
    LdSPnn: ("LD SP,nn", 0x31, 12),

    // LD SP,HL
    //
    // Put HL into Stack Pointer (SP)
    LdSPHL: ("LD SP,HL", 0xF9, 8),

    // LDHL SP,n
    //
    // Put SP + n effective address into HL
    //
    // n = one byte signed immediate value
    //
    // Flags:
    // Z - Reset
    // N - Reset
    // H - Set or reset according to operation
    // C - Set or reset according to operation
    LDHLSPn: ("LDHL SP,n", 0xF8, 12),

    // LD (nn),SP
    //
    // Put Stack Pointer (SP) at address nn
    //
    // nn = two byte immediate address.
    LDnnSP: ("LD (nn),SP", 0x08, 20),

    // PUSH nn
    //
    // Push register pair nn onto stack.
    // Decrement Stack Pointer (SP) twice.
    //
    // nn = AF,BC,DE,HL
    PushAF: ("PUSH AF", 0xF5, 16),
    PushBC: ("PUSH BC", 0xC5, 16),
    PushDE: ("PUSH DE", 0xD5, 16),
    PushHL: ("PUSH HL", 0xE5, 16),

    // POP nn
    //
    // Pop two bytes off stack into register pair nn.
    // Increment Stack Pointer (SP) twice.
    //
    // nn = AF,BC,DE,HL
    PopAF: ("POP AF", 0xF1, 12),
    PopBC: ("POP BC", 0xC1, 12),
    PopDE: ("POP DE", 0xD1, 12),
    PopHL: ("POP HL", 0xE1, 12),

    // ADD A,n
    //
    // Add n to A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Set if carry from bit 3
    // C - Set if carry from bit 7
    AddAA:  ("ADD A,A",    0x87, 4),
    AddAB:  ("ADD A,B",    0x80, 4),
    AddAC:  ("ADD A,C",    0x81, 4),
    AddAD:  ("ADD A,D",    0x82, 4),
    AddAE:  ("ADD A,E",    0x83, 4),
    AddAH:  ("ADD A,H",    0x84, 4),
    AddAL:  ("ADD A,L",    0x85, 4),
    AddAHL: ("ADD A,(HL)", 0x86, 8),
    AddAx:  ("ADD A,#",    0xC6, 8),

    // ADC A,n
    //
    // Add n + Carry flag to A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Set if carry from bit 3
    // C - Set if carry from bit 7
    AdcAA:  ("ADC A,A",    0x8F, 4),
    AdcAB:  ("ADC A,B",    0x88, 4),
    AdcAC:  ("ADC A,C",    0x89, 4),
    AdcAD:  ("ADC A,D",    0x8A, 4),
    AdcAE:  ("ADC A,E",    0x8B, 4),
    AdcAH:  ("ADC A,H",    0x8C, 4),
    AdcAL:  ("ADC A,L",    0x8D, 4),
    AdcAHL: ("ADC A,(HL)", 0x8E, 8),
    AdcAx:  ("ADC A,#",    0xCE, 8),

    // SUB n
    //
    // Subtract n from A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Set
    // H - Set if no borrow from bit 4
    // C - Set if no borrow
    SubAA:  ("SUB A,A",    0x97, 4),
    SubAB:  ("SUB A,B",    0x90, 4),
    SubAC:  ("SUB A,C",    0x91, 4),
    SubAD:  ("SUB A,D",    0x92, 4),
    SubAE:  ("SUB A,E",    0x93, 4),
    SubAH:  ("SUB A,H",    0x94, 4),
    SubAL:  ("SUB A,L",    0x95, 4),
    SubAHL: ("SUB A,(HL)", 0x96, 8),
    SubAx:  ("SUB A,#",    0xD6, 8),

    // SBC A,n
    //
    // Subtract n + Carry flag from A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Set
    // H - Set if no borrow from bit 4
    // C - Set if no borrow
    SbcAA:  ("SBC A,A",    0x9F, 4),
    SbcAB:  ("SBC A,B",    0x98, 4),
    SbcAC:  ("SBC A,C",    0x99, 4),
    SbcAD:  ("SBC A,D",    0x9A, 4),
    SbcAE:  ("SBC A,E",    0x9B, 4),
    SbcAH:  ("SBC A,H",    0x9C, 4),
    SbcAL:  ("SBC A,L",    0x9D, 4),
    SbcAHL: ("SBC A,(HL)", 0x9E, 8),

    // AND n
    //
    // Logically AND n with A, result in A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Set
    // C - Reset
    AndAA:  ("AND A,A",    0xA7, 4),
    AndAB:  ("AND A,B",    0xA0, 4),
    AndAC:  ("AND A,C",    0xA1, 4),
    AndAD:  ("AND A,D",    0xA2, 4),
    AndAE:  ("AND A,E",    0xA3, 4),
    AndAH:  ("AND A,H",    0xA4, 4),
    AndAL:  ("AND A,L",    0xA5, 4),
    AndAHL: ("AND A,(HL)", 0xA6, 8),
    AndAx:  ("AND A,#",    0xE6, 8),

    // OR n
    //
    // Logical OR n with register A, result in A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Reset
    OrAA:  ("OR A,A",    0xB7, 4),
    OrAB:  ("OR A,B",    0xB0, 4),
    OrAC:  ("OR A,C",    0xB1, 4),
    OrAD:  ("OR A,D",    0xB2, 4),
    OrAE:  ("OR A,E",    0xB3, 4),
    OrAH:  ("OR A,H",    0xB4, 4),
    OrAL:  ("OR A,L",    0xB5, 4),
    OrAHL: ("OR A,(HL)", 0xB6, 8),
    OrAx:  ("OR A,#",    0xF6, 8),

    // XOR n
    //
    // Logical exclusive OR n with register A, result in A
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Reset
    XorAA:  ("XOR A,A",    0xAF, 4),
    XorAB:  ("XOR A,B",    0xA8, 4),
    XorAC:  ("XOR A,C",    0xA9, 4),
    XorAD:  ("XOR A,D",    0xAA, 4),
    XorAE:  ("XOR A,E",    0xAB, 4),
    XorAH:  ("XOR A,H",    0xAC, 4),
    XorAL:  ("XOR A,L",    0xAD, 4),
    XorAHL: ("XOR A,(HL)", 0xAE, 8),
    XorAx:  ("XOR A,#",    0xEE, 8),

    // CP n
    //
    // Compare A with n. This is basically an A - n subtraction
    // instruction but the results are thrown away
    //
    // n = A,B,C,D,E,H,L,(HL),#
    //
    // Flags:
    // Z - Set if result is zero. (Set if A = n)
    // N - Set
    // H - Set if no borrow from bit 4
    // C - Set for no borrow. (Set if A < n)
    CpAA:  ("CP A,A",    0xBF, 4),
    CpAB:  ("CP A,B",    0xB8, 4),
    CpAC:  ("CP A,C",    0xB9, 4),
    CpAD:  ("CP A,D",    0xBA, 4),
    CpAE:  ("CP A,E",    0xBB, 4),
    CpAH:  ("CP A,H",    0xBC, 4),
    CpAL:  ("CP A,L",    0xBD, 4),
    CpAHL: ("CP A,(HL)", 0xBE, 8),
    CpAx:  ("CP A,#",    0xFE, 8),

    // INC n
    //
    // Increment register n
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero.
    // N - Reset
    // H - Set if carry from bit 3
    // C - Not affected
    IncA:   ("INC A",    0x3C, 4),
    IncB:   ("INC B",    0x04, 4),
    IncC:   ("INC C",    0x0C, 4),
    IncD:   ("INC D",    0x14, 4),
    IncE:   ("INC E",    0x1C, 4),
    IncH:   ("INC H",    0x24, 4),
    IncL:   ("INC L",    0x2C, 4),
    IncHLp: ("INC (HL)", 0x34, 12),

    // DEC n
    //
    // Decrement register n
    //
    // n = A,B,C,D,E,H,L,(HL)
    //
    // Flags:
    // Z - Set if result is zero.
    // N - Set
    // H - Set if no borrow from bit 4
    // C - Not affected
    DecA:   ("DEC A",    0x3D, 4),
    DecB:   ("DEC B",    0x05, 4),
    DecC:   ("DEC C",    0x0D, 4),
    DecD:   ("DEC D",    0x15, 4),
    DecE:   ("DEC E",    0x1D, 4),
    DecH:   ("DEC H",    0x25, 4),
    DecL:   ("DEC L",    0x2D, 4),
    DecHLp: ("DEC (HL)", 0x35, 12),

    // ADD HL,n
    //
    // Add n to HL
    //
    // n = BC,DE,HL,SP
    //
    // Flags:
    // Z - Not affected
    // N - Reset
    // H - Set if carry from bit 11
    // C - Set if carry from bit 15
    AddHLBC: ("ADD HL,BC", 0x09, 8),
    AddHLDE: ("ADD HL,DE", 0x19, 8),
    AddHLHL: ("ADD HL,HL", 0x29, 8),
    AddHLSP: ("ADD HL,SP", 0x39, 8),

    // ADD SP,n
    //
    // Add n to Stack Pointer (SP)
    //
    // n = one byte signed immediate value (#)
    //
    // Flags:
    // Z - Reset
    // N - Reset
    // H - Set or reset according to operation
    // C - Set or reset according to operation
    AddSPx: ("ADD SP,#", 0xE8, 16),

    // INC nn
    //
    // Increment register nn
    //
    // nn = BC,DE,HL,SP
    IncBC: ("INC BC", 0x03, 8),
    IncDE: ("INC DE", 0x13, 8),
    IncHL: ("INC HL", 0x23, 8),
    IncSP: ("INC SP", 0x33, 8),

    // DEC nn
    //
    // Decrement register nn
    //
    // nn = BC,DE,HL,SP
    DecBC: ("DEC BC", 0x0B, 8),
    DecDE: ("DEC DE", 0x1B, 8),
    DecHL: ("DEC HL", 0x2B, 8),
    DecSP: ("DEC SP", 0x3B, 8),

    // DAA
    //
    // Decimal adjust register A
    // This instruction adjusts register A so that the
    // correct representation of Binary Coded Decimal (BCD)
    // is obtained
    //
    // Flags:
    // Z - Set if register A is zero
    // N - Not affected
    // H - Reset
    // C - Set or reset according to operation
    Daa: ("DAA", 0x27, 4),

    // CPL
    //
    // Complement A register. (Flip all bits).
    //
    // Flags:
    // Z - Not affected
    // N - Set
    // H - Set
    // C - Not affected
    Cpl: ("CPL", 0x2F, 4),

    // CCF
    //
    // Complement carry flag.
    // If C flag is set, then reset it.
    // If C flag is reset, then set it.
    //
    // Flags:
    // Z - Not affected
    // N - Reset
    // H - Reset
    // C - Complemented
    Ccf: ("CCF", 0x3F, 4),

    // SCF
    //
    // Set carry flag.
    //
    // Flags:
    // Z - Not affected
    // N - Reset
    // H - Reset
    // C - Set
    Scf: ("SCF", 0x37, 4),

    // NOP
    //
    // No operation
    Nop: ("NOP", 0x00, 4),

    // HALT
    //
    // Power down CPU until an interrupt occurs. Use this
    // when ever possible to reduce energy consumption
    Halt: ("HALT", 0x76, 4),

    // STOP
    //
    // Halt CPU & LCD display until button pressed.
    Stop: ("STOP", 0x10, 4),

    // DI
    //
    // This instruction disables interrupts but not
    // immediately. Interrupts are disabled after
    // the instruction after DI is executed.
    Di: ("DI", 0xF3, 4),

    // EI
    //
    // Enable interrupts. This instruction enables interrupts
    // but not immediately. Interrupts are enabled after
    // the instruction after EI is executed.
    Ei: ("EI", 0xFB, 4),

    // RLCA
    //
    // Rotate A left. Old bit 7 to Carry flag.
    //
    // Flags:
    // Z - Set if result is Zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 7 data.
    Rlca: ("RLCA", 0x07, 4),

    // RLA
    //
    // Rotate A left thorugh Carry flag.
    //
    // Flags:
    // Z - Set if result is zero
    // N - Reset
    // H - Reset
    // C - Contains old bit 7 data
    Rla: ("RLA", 0x17, 4),

    // RRCA
    //
    // Rotate A right. Old bit 0 to Carry flag.
    //
    // Flags:
    // Z - Set if result is zero.
    // N - Reset
    // H - Reset
    // C - Contains old bit 0 data
    Rrca: ("RRCA", 0x0F, 4),

    // RRA
    //
    // Rotate A right through Carry flag.
    //
    // Flags:
    // Z - Set if result is zero.
    // N - Reset
    // H - Reset
    // C - Contains old bit 0 data
    Rra: ("Rra", 0x1F, 4),

    // JP nn
    //
    // Jump to address nn
    //
    // nn = two bytes immediate value. (LS byte first)
    Jp: ("JP", 0xC3, 12),

    // JP cc,nn
    //
    // Jump to address n if following condition is true:
    // cc = NZ, Jump if Z flag is reset
    // cc = Z,  Jump if Z flag is set
    // cc = NC, Jump if C flag is reset
    // cc = Z,  Jump if C flag is set
    //
    // nn = two byte immediate value. (LS byte first)
    JpNZnn: ("JP NZ,nn", 0xC2, 12),
    JpZnn:  ("JP Z,nn",  0xCA, 12),
    JpNCnn: ("JP NC,nn", 0xD2, 12),
    JpCnn:  ("JP C,nn",  0xDA, 12),

    // JP (HL)
    //
    // Jump to address contained in HL.
    JpHL: ("JP (HL)", 0xE9, 4),

    // JR n
    //
    // Add n to current address and jump to it
    //
    // n = one byte signed immediate value
    Jrn: ("JR n", 0x18, 8),

    // JR cc,n
    //
    // If following condition is true then add n to current
    // address and jump to it
    //
    // n = one byte signed immediate value
    // cc = NZ, Jump if Z flag is reset
    // cc = Z,  Jump if Z flag is set
    // cc = NC, Jump if C flag is reset
    // cc = C,  Jump if C flag is set
    JrNZn: ("JR NZ,n", 0x20, 8),
    JrZn:  ("JR Z,n",  0x28, 8),
    JrNCn: ("JR NC,n", 0x30, 8),
    JrCn:  ("JR C,n",  0x38, 8),

    // CALL nn
    //
    // Push address of next instruction onto stack and then
    // jump to address nn
    //
    // nn = two byte immediate value. (LS byte first.)
    Call: ("CALL nn", 0xCD, 12),

    // CALL cc,nn
    //
    // Call address n if following condition is true:
    // cc = NZ, Jump if Z flag is reset
    // cc = Z,  Jump if Z flag is set
    // cc = NC, Jump if C flag is reset
    // cc = C,  Jump if C flag is set
    //
    // nn = two byte immediate value. (LS byte first.)
    CallNZnn: ("CALL NZ,nn", 0xC4, 12),
    CallZnn:  ("CALL Z,nn",  0xCC, 12),
    CallNCnn: ("CALL NC,nn", 0xD4, 12),
    CallCnn:  ("CALL C,nn",  0xDC, 12),

    // RST n
    //
    // Push present address onto stack
    // Jump to address $0000 + n
    //
    // n = $00, $08, $10, $18, $20, $28, $30, $38
    Rst00: ("RST 00H", 0xC7, 32),
    Rst08: ("RST 08H", 0xCF, 32),
    Rst10: ("RST 10H", 0xD7, 32),
    Rst18: ("RST 18H", 0xDF, 32),
    Rst20: ("RST 20H", 0xE7, 32),
    Rst28: ("RST 28H", 0xEF, 32),
    Rst30: ("RST 30H", 0xF7, 32),
    Rst38: ("RST 38H", 0xFF, 32),

    // RET
    //
    // Pop two bytes from stack and jump to that address
    Ret: ("RET", 0xC9, 8),

    // RET cc
    //
    // Return if following condition is true
    // cc = NZ, Jump if Z flag is reset
    // cc = Z,  Jump if Z flag is set
    // cc = NC, Jump if C flag is reset
    // cc = C,  Jump if C flag is set
    RetNZ: ("RET NZ", 0xC0, 8),
    RetZ:  ("RET Z",  0xC8, 8),
    RetNC: ("RET NC", 0xD0, 8),
    RetC:  ("RET C",  0xD8, 8),

    // RETI
    //
    // Pop two bytes from stack and jump to that address then
    // enable interrupts
    Reti: ("RETI", 0xD9, 8)
);
