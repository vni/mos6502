/** USEFUL LINKS
    https://www.pagetable.com/c64ref/6502/?tab=2
    http://www.emulator101.com/6502-addressing-modes.html
    https://www.masswerk.at/6502/6502_instruction_set.html
    https://web.archive.org/web/20221112230813if_/http://archive.6502.org/books/mcs6500_family_programming_manual.pdf
*/

/** TODO
    - make Cpu reference to memory, not owning it
      (it may speed up tests a little bit)
    - make all the tests look the same: all have some _t function
      All have ready memory, no memory modification during tests,
      no pre-calculated addresses passed to test
    - add separate functions for different addressing modes
    - use update_*() all over the code
    - update tests for AND, ORA, EOR to have common code for different addressing modes
    - crossing page boundary (page zero addressing) ? What does it mean ?
    - tests: make the cpu to start execution from some `org`, not from 0x00.
      it will allow to test the zero_page overlap (addresses 0xff and 0x00).
    - make all memory addressing aux functions return not the reference but address (usize)
    - add aux function to work with stack (push, push_2b, pop, pop_2b)
    - add tests for CONTROL instructions
    - run test roms for 6502
    - in all the instructions add table of correspondance of addressing mode and pc increment
    - move cpu.pc update from instructions::functions() to step()
    - unify addressing instructions to return the usize offset in cpu.memory instead of a reference
 */


const MEM_SZ: usize = 65_536;

#[derive(Debug, PartialEq)]
pub struct Cpu {
    a: u8,   // accumulator
    x: u8,   // x index register
    y: u8,   // y index register
    pc: u16, // program counter
    s: u8,   // stack
    p: u8,   // flags

    memory: [u8; MEM_SZ], // Silly, but currently memory is part of the processor
}

#[allow(non_snake_case)] // ?? FIXME ??
mod Flags {
    #[allow(non_upper_case_globals)]
    pub(crate) const N_Negative: u8 = 0x80;

    #[allow(non_upper_case_globals)]
    pub(crate) const V_Overflow: u8 = 0x40;

    // the 0x20 bit is unused
    //
    #[allow(dead_code)] // FIXME
    #[allow(non_upper_case_globals)]
    pub(crate) const B_Break: u8 = 0x10;

    #[allow(non_upper_case_globals)]
    pub(crate) const D_Decimal: u8 = 0x08;

    #[allow(non_upper_case_globals)]
    pub(crate) const I_InterruptDisable: u8 = 0x04;

    #[allow(non_upper_case_globals)]
    pub(crate) const Z_Zero: u8 = 0x02;

    #[allow(non_upper_case_globals)]
    pub(crate) const C_Carry: u8 = 0x01;
}

#[derive(PartialEq, Copy, Clone)]
pub enum AddressingMode {
    Implied,
    Accumulator,
    Immediate,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    AbsoluteIndirect,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
    ZeroPageXIndirect,
    ZeroPageIndirectY,
    Relative, // ???
}

mod opcodes {
    // load
    pub const LDA_A9: u8 = 0xA9;
    pub const LDA_AD: u8 = 0xAD;
    pub const LDA_BD: u8 = 0xBD;
    pub const LDA_B9: u8 = 0xB9;
    pub const LDA_A5: u8 = 0xA5;
    pub const LDA_B5: u8 = 0xB5;
    pub const LDA_A1: u8 = 0xA1;
    pub const LDA_B1: u8 = 0xB1;

    pub const LDX_A2: u8 = 0xA2;
    pub const LDX_AE: u8 = 0xAE;
    pub const LDX_BE: u8 = 0xBE;
    pub const LDX_A6: u8 = 0xA6;
    pub const LDX_B6: u8 = 0xB6;

    pub const LDY_A0: u8 = 0xA0;
    pub const LDY_AC: u8 = 0xAC;
    pub const LDY_BC: u8 = 0xBC;
    pub const LDY_A4: u8 = 0xA4;
    pub const LDY_B4: u8 = 0xB4;

    pub const STA_8D: u8 = 0x8D;
    pub const STA_9D: u8 = 0x9D;
    pub const STA_99: u8 = 0x99;
    pub const STA_85: u8 = 0x85;
    pub const STA_95: u8 = 0x95;
    pub const STA_81: u8 = 0x81;
    pub const STA_91: u8 = 0x91;

    pub const STX_8E: u8 = 0x8E;
    pub const STX_86: u8 = 0x86;
    pub const STX_96: u8 = 0x96;

    pub const STY_8C: u8 = 0x8C;
    pub const STY_84: u8 = 0x84;
    pub const STY_94: u8 = 0x94;

    // transfer
    pub const TAX_AA: u8 = 0xAA;
    pub const TAY_A8: u8 = 0xA8;
    pub const TSX_BA: u8 = 0xBA;
    pub const TXA_8A: u8 = 0x8A;
    pub const TXS_9A: u8 = 0x9A;
    pub const TYA_98: u8 = 0x98;

    // stack
    pub const PHA_48: u8 = 0x48;
    pub const PHP_08: u8 = 0x08;
    pub const PLA_68: u8 = 0x68;
    pub const PLP_28: u8 = 0x28;

    // shift
    pub const ASL_0A: u8 = 0x0A;
    pub const ASL_0E: u8 = 0x0E;
    pub const ASL_1E: u8 = 0x1E;
    pub const ASL_06: u8 = 0x06;
    pub const ASL_16: u8 = 0x16;

    pub const LSR_4A: u8 = 0x4A;
    pub const LSR_4E: u8 = 0x4E;
    pub const LSR_5E: u8 = 0x5E;
    pub const LSR_46: u8 = 0x46;
    pub const LSR_56: u8 = 0x56;

    pub const ROL_2A: u8 = 0x2A;
    pub const ROL_2E: u8 = 0x2E;
    pub const ROL_3E: u8 = 0x3E;
    pub const ROL_26: u8 = 0x26;
    pub const ROL_36: u8 = 0x36;

    pub const ROR_6A: u8 = 0x6A;
    pub const ROR_6E: u8 = 0x6E;
    pub const ROR_7E: u8 = 0x7E;
    pub const ROR_66: u8 = 0x66;
    pub const ROR_76: u8 = 0x76;

    // logic
    pub const AND_29: u8 = 0x29;
    pub const AND_2D: u8 = 0x2D;
    pub const AND_3D: u8 = 0x3D;
    pub const AND_39: u8 = 0x39;
    pub const AND_25: u8 = 0x25;
    pub const AND_35: u8 = 0x35;
    pub const AND_21: u8 = 0x21;
    pub const AND_31: u8 = 0x31;

    pub const BIT_2C: u8 = 0x2C;
    pub const BIT_24: u8 = 0x24;

    pub const EOR_49: u8 = 0x49;
    pub const EOR_4D: u8 = 0x4D;
    pub const EOR_5D: u8 = 0x5D;
    pub const EOR_59: u8 = 0x59;
    pub const EOR_45: u8 = 0x45;
    pub const EOR_55: u8 = 0x55;
    pub const EOR_41: u8 = 0x41;
    pub const EOR_51: u8 = 0x51;

    pub const ORA_09: u8 = 0x09;
    pub const ORA_0D: u8 = 0x0D;
    pub const ORA_1D: u8 = 0x1D;
    pub const ORA_19: u8 = 0x19;
    pub const ORA_05: u8 = 0x05;
    pub const ORA_15: u8 = 0x15;
    pub const ORA_01: u8 = 0x01;
    pub const ORA_11: u8 = 0x11;

    // arith
    pub const ADC_69: u8 = 0x69;
    pub const ADC_6D: u8 = 0x6D;
    pub const ADC_7D: u8 = 0x7D;
    pub const ADC_79: u8 = 0x79;
    pub const ADC_65: u8 = 0x65;
    pub const ADC_75: u8 = 0x75;
    pub const ADC_61: u8 = 0x61;
    pub const ADC_71: u8 = 0x71;

    pub const CMP_C9: u8 = 0xC9;
    pub const CMP_CD: u8 = 0xCD;
    pub const CMP_DD: u8 = 0xDD;
    pub const CMP_D9: u8 = 0xD9;
    pub const CMP_C5: u8 = 0xC5;
    pub const CMP_D5: u8 = 0xD5;
    pub const CMP_C1: u8 = 0xC1;
    pub const CMP_D1: u8 = 0xD1;

    pub const CPX_E0: u8 = 0xE0;
    pub const CPX_EC: u8 = 0xEC;
    pub const CPX_E4: u8 = 0xE4;

    pub const CPY_C0: u8 = 0xC0;
    pub const CPY_CC: u8 = 0xCC;
    pub const CPY_C4: u8 = 0xC4;

    pub const SBC_E9: u8 = 0xE9;
    pub const SBC_ED: u8 = 0xED;
    pub const SBC_FD: u8 = 0xFD;
    pub const SBC_F9: u8 = 0xF9;
    pub const SBC_E5: u8 = 0xE5;
    pub const SBC_F5: u8 = 0xF5;
    pub const SBC_E1: u8 = 0xE1;
    pub const SBC_F1: u8 = 0xF1;

    // increment
    pub const DEC_CE: u8 = 0xCE;
    pub const DEC_DE: u8 = 0xDE;
    pub const DEC_C6: u8 = 0xC6;
    pub const DEC_D6: u8 = 0xD6;

    pub const DEX_CA: u8 = 0xCA;
    pub const DEY_88: u8 = 0x88;

    pub const INC_EE: u8 = 0xEE;
    pub const INC_FE: u8 = 0xFE;
    pub const INC_E6: u8 = 0xE6;
    pub const INC_F6: u8 = 0xF6;

    pub const INX_E8: u8 = 0xE8;
    pub const INY_C8: u8 = 0xC8;

    // control
    pub const BRK_00: u8 = 0x00;
    pub const JMP_4C: u8 = 0x4C;
    pub const JMP_6C: u8 = 0x6C;
    pub const JSR_20: u8 = 0x20;
    pub const RTI_40: u8 = 0x40;
    pub const RTS_60: u8 = 0x60;

    // branch
    pub const BCC_90: u8 = 0x90;
    pub const BCS_B0: u8 = 0xB0;
    pub const BEQ_F0: u8 = 0xF0;
    pub const BMI_30: u8 = 0x30;
    pub const BNE_D0: u8 = 0xD0;
    pub const BPL_10: u8 = 0x10;
    pub const BVC_50: u8 = 0x50;
    pub const BVS_70: u8 = 0x70;

    // flags
    pub const CLC_18: u8 = 0x18;
    pub const CLD_D8: u8 = 0xD8;
    pub const CLI_58: u8 = 0x58;
    pub const CLV_B8: u8 = 0xB8;
    pub const SEC_38: u8 = 0x38;
    pub const SED_F8: u8 = 0xF8;
    pub const SEI_78: u8 = 0x78;

    // nop
    pub const NOP_EA: u8 = 0xEA;
}

fn addressing_mode_pc_advance(mode: AddressingMode) -> u16 {
    // FIXME: change it to indexable table
    match mode {
        AddressingMode::Implied => 0,
        AddressingMode::Accumulator => 0,
        AddressingMode::Immediate => 1,
        AddressingMode::Absolute => 2,
        AddressingMode::AbsoluteX => 2,
        AddressingMode::AbsoluteY => 2,
        AddressingMode::AbsoluteIndirect => 2,
        AddressingMode::ZeroPage => 1,
        AddressingMode::ZeroPageX => 1,
        AddressingMode::ZeroPageY => 1,
        AddressingMode::ZeroPageXIndirect => 1,
        AddressingMode::ZeroPageIndirectY => 1,
        AddressingMode::Relative => todo!(),
    }
}

//
// instructions
//
mod instructions {
    use super::*;

    //
    // load
    //
    pub fn lda(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => cpu.a = cpu.memory[cpu._immediate()],
            AddressingMode::Absolute => cpu.a = cpu.memory[cpu._absolute()],
            AddressingMode::AbsoluteX => cpu.a = cpu.memory[cpu._absolute_x()],
            AddressingMode::AbsoluteY => cpu.a = cpu.memory[cpu._absolute_y()],
            AddressingMode::ZeroPage => cpu.a = cpu.memory[cpu._zero_page()],
            AddressingMode::ZeroPageX => cpu.a = cpu.memory[cpu._zero_page_x()],
            AddressingMode::ZeroPageXIndirect => cpu.a = cpu.memory[cpu._zero_page_x_indirect()],
            AddressingMode::ZeroPageIndirectY => cpu.a = cpu.memory[cpu._zero_page_indirect_y()],
            _ => unimplemented!("bad addressing mode for the LDA instruction"),
        }

        cpu.update_negative(cpu.a & 0x80 != 0);
        cpu.update_zero(cpu.a == 0);

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    pub fn ldx(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => cpu.x = cpu.memory[cpu._immediate()],
            AddressingMode::Absolute => cpu.x = cpu.memory[cpu._absolute()],
            AddressingMode::AbsoluteY => cpu.x = cpu.memory[cpu._absolute_y()],
            AddressingMode::ZeroPage => cpu.x = cpu.memory[cpu._zero_page()],
            AddressingMode::ZeroPageY => cpu.x = cpu.memory[cpu._zero_page_y()],
            _ => unimplemented!("bad addressing mode for the LDX instruction"),
        }

        cpu.update_negative(cpu.x & 0x80 != 0);
        cpu.update_zero(cpu.x == 0);

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    pub fn ldy(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Immediate => cpu.y = cpu.memory[cpu._immediate()],
            AddressingMode::Absolute => cpu.y = cpu.memory[cpu._absolute()],
            AddressingMode::AbsoluteX => cpu.y = cpu.memory[cpu._absolute_x()],
            AddressingMode::ZeroPage => cpu.y = cpu.memory[cpu._zero_page()],
            AddressingMode::ZeroPageX => cpu.y = cpu.memory[cpu._zero_page_x()],
            _ => unimplemented!("bad addressing mode for the LDY instruction"),
        }

        cpu.update_negative(cpu.y & 0x80 != 0);
        cpu.update_zero(cpu.y == 0);

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    pub fn sta(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Absolute => cpu.memory[cpu._absolute()] = cpu.a,
            AddressingMode::AbsoluteX => cpu.memory[cpu._absolute_x()] = cpu.a,
            AddressingMode::AbsoluteY => cpu.memory[cpu._absolute_y()] = cpu.a,
            AddressingMode::ZeroPage => cpu.memory[cpu._zero_page()] = cpu.a,
            AddressingMode::ZeroPageX => cpu.memory[cpu._zero_page_x()] = cpu.a,
            AddressingMode::ZeroPageXIndirect => cpu.memory[cpu._zero_page_x_indirect()] = cpu.a,
            AddressingMode::ZeroPageIndirectY => cpu.memory[cpu._zero_page_indirect_y()] = cpu.a,
            _ => unimplemented!("bad addressing mode for the STX instruction"),
        }

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    pub fn stx(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Absolute => cpu.memory[cpu._absolute()] = cpu.x,
            AddressingMode::ZeroPage => cpu.memory[cpu._zero_page()] = cpu.x,
            AddressingMode::ZeroPageY => cpu.memory[cpu._zero_page_y()] = cpu.x,
            _ => unimplemented!("bad addressing mode for the STX instruction"),
        }

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    pub fn sty(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Absolute => cpu.memory[cpu._absolute()] = cpu.y,
            AddressingMode::ZeroPage => cpu.memory[cpu._zero_page()] = cpu.y,
            AddressingMode::ZeroPageX => cpu.memory[cpu._zero_page_x()] = cpu.y,
            _ => unimplemented!("bad addressing mode for the STY instruction"),
        }

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    //
    // trans
    //
    pub fn tax(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Implied => cpu.x = cpu.a,
            _ => unimplemented!("bad addressing mode for the TAX instruction"),
        }

        cpu.update_negative(cpu.x & 0x80 != 0);
        cpu.update_zero(cpu.x == 0);

        cpu.pc += addressing_mode_pc_advance(mode); // UNNECESSARY: SHOULD ALWAIS BE 0
    }

    pub fn tay(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Implied => cpu.y = cpu.a,
            _ => unimplemented!("bad addressing mode for the TAY instruction"),
        }

        cpu.update_negative(cpu.y & 0x80 != 0);
        cpu.update_zero(cpu.y == 0);

        cpu.pc += addressing_mode_pc_advance(mode); // UNNECESSARY: SHOULD ALWAIS BE 0
    }

    pub fn tsx(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Implied => cpu.x = cpu.s,
            _ => unimplemented!("bad addressing mode for the TSX instruction"),
        }

        cpu.update_negative(cpu.x & 0x80 != 0);
        cpu.update_zero(cpu.x == 0);

        cpu.pc += addressing_mode_pc_advance(mode); // UNNECESSARY: SHOULD ALWAIS BE 0
    }

    pub fn txa(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Implied => cpu.a = cpu.x,
            _ => unimplemented!("bad addressing mode for the TXA instruction"),
        }

        cpu.update_negative(cpu.a & 0x80 != 0);
        cpu.update_zero(cpu.a == 0);

        cpu.pc += addressing_mode_pc_advance(mode); // UNNECESSARY: SHOULD ALWAIS BE 0
    }

    pub fn txs(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Implied => cpu.s = cpu.x,
            _ => unimplemented!("bad addressing mode for the TXS instruction"),
        }

        cpu.pc += addressing_mode_pc_advance(mode); // UNNECESSARY: SHOULD ALWAIS BE 0
    }

    pub fn tya(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Implied => cpu.a = cpu.y,
            _ => unimplemented!("bad addressing mode for the TYA instruction"),
        }

        cpu.update_negative(cpu.a & 0x80 != 0);
        cpu.update_zero(cpu.a == 0);

        cpu.pc += addressing_mode_pc_advance(mode); // UNNECESSARY: SHOULD ALWAIS BE 0
    }

    //
    // stack
    //
    pub fn pha(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Implied => {
                cpu.memory[0x100 + cpu.s as usize] = cpu.a;
                cpu.s -= 1;
            }
            _ => unimplemented!("bad addressing mode for the PHA instruction"),
        }

        cpu.pc += addressing_mode_pc_advance(mode); // UNNECESSARY: SHOULD ALWAIS BE 0
    }

    pub fn php(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Implied => {
                cpu.memory[0x100 + cpu.s as usize] = cpu.p;
                cpu.s -= 1;
            }
            _ => unimplemented!("bad addressing mode for the PHP instruction"),
        }

        cpu.pc += addressing_mode_pc_advance(mode); // UNNECESSARY: SHOULD ALWAIS BE 0
    }

    pub fn pla(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Implied => {
                cpu.s += 1;
                cpu.a = cpu.memory[0x100 + cpu.s as usize];
            }
            _ => unimplemented!("bad addressing mode for the PLA instruction"),
        }

        // FIXME: do this flags update is really needed here?
        cpu.update_negative(cpu.a & 0x80 != 0);
        cpu.update_zero(cpu.a == 0);

        cpu.pc += addressing_mode_pc_advance(mode); // UNNECESSARY: SHOULD ALWAIS BE 0
    }

    pub fn plp(cpu: &mut Cpu, mode: AddressingMode) {
        match mode {
            AddressingMode::Implied => {
                cpu.s += 1;
                cpu.p = cpu.memory[0x100 + cpu.s as usize];
            }
            _ => unimplemented!("bad addressing mode for the PLP instruction"),
        }

        cpu.pc += addressing_mode_pc_advance(mode); // UNNECESSARY: SHOULD ALWAIS BE 0
    }

    //
    // shift
    //
    pub fn asl(cpu: &mut Cpu, mode: AddressingMode) {
        fn _asl(cpu: &mut Cpu, mut val: u8) -> u8 {
            cpu.update_carry(val & 0x80 != 0);
            val <<= 1;
            cpu.update_negative(val & 0x80 != 0);
            cpu.update_zero(val == 0);
            val
        }

        if mode == AddressingMode::Accumulator {
                cpu.a = _asl(cpu, cpu.a);
        } else {
            let addr = match mode {
                AddressingMode::Absolute => cpu._absolute(),
                AddressingMode::AbsoluteX => cpu._absolute_x(),
                AddressingMode::ZeroPage => cpu._zero_page(),
                AddressingMode::ZeroPageX => cpu._zero_page_x(),
                _ => unimplemented!("bad addressing mode for the ASL instruction"),
            };

            cpu.memory[addr] = _asl(cpu, cpu.memory[addr]);
        }

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // SHIFT - LSR - Logic Shift Right
    pub fn lsr(cpu: &mut Cpu, mode: AddressingMode) {
        fn _lsr(cpu: &mut Cpu, mut val: u8) -> u8 {
            cpu.update_carry(val & 1 != 0);
            val >>= 1;
            cpu.update_negative(false);
            cpu.update_zero(val == 0);
            val
        }

        if mode == AddressingMode::Accumulator { // LSR A
                cpu.a = _lsr(cpu, cpu.a);
        } else {
            let addr = match mode {
                AddressingMode::Absolute => cpu._absolute(), // LSR $nnnn
                AddressingMode::AbsoluteX => cpu._absolute_x(), // LSR $nnnn,X
                AddressingMode::ZeroPage => cpu._zero_page(), // LSR $nn
                AddressingMode::ZeroPageX => cpu._zero_page_x(), // LSR $nn,X
                _ => unimplemented!("bad addressing mode for the LSR instruction"),
            };

            cpu.memory[addr] = _lsr(cpu, cpu.memory[addr]);
        }

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // SHIFT - ROL - Rotate Left
    pub fn rol(cpu: &mut Cpu, mode: AddressingMode) {
        fn _rol(cpu: &mut Cpu, mut val: u8) -> u8 {
            let prev_carry = cpu.is_carry();
            cpu.update_carry(val & 0x80 != 0);
            val <<= 1;
            if prev_carry {
                val |= 1;
            }
            cpu.update_negative(val & 0x80 != 0);
            cpu.update_zero(val == 0);
            val
        }

        if mode == AddressingMode::Accumulator { // ROL A
                cpu.a = _rol(cpu, cpu.a);
        } else {
            let addr = match mode {
                AddressingMode::Absolute => cpu._absolute(), // ROL $nnnn
                AddressingMode::AbsoluteX => cpu._absolute_x(), // ROL $nnnn,X
                AddressingMode::ZeroPage => cpu._zero_page(), // ROL $nn
                AddressingMode::ZeroPageX => cpu._zero_page_x(), // ROL $nn,X
                _ => unimplemented!("bad addressing mode for the ROL instruction"),
            };

            cpu.memory[addr] = _rol(cpu, cpu.memory[addr]);
        }

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // SHIFT - ROR - Rotate Right
    pub fn ror(cpu: &mut Cpu, mode: AddressingMode) {
        fn _ror(cpu: &mut Cpu, mut val: u8) -> u8 {
            let prev_carry = cpu.is_carry();
            cpu.update_carry(val & 1 != 0);
            val >>= 1;
            if prev_carry {
                val |= 0x80;
            }
            cpu.update_negative(val & 0x80 != 0);
            cpu.update_zero(val == 0);
            val
        }

        if mode == AddressingMode::Accumulator { // ROR A
                cpu.a = _ror(cpu, cpu.a);
        } else {
            let addr = match mode {
                AddressingMode::Absolute => cpu._absolute(), // ROR $nnnn
                AddressingMode::AbsoluteX => cpu._absolute_x(), // ROR $nnnn,X
                AddressingMode::ZeroPage => cpu._zero_page(), // ROR $nn
                AddressingMode::ZeroPageX => cpu._zero_page_x(), // ROR $nn,X
                _ => unimplemented!("bad addressing mode for the ROR instruction"),
            };

            cpu.memory[addr] = _ror(cpu, cpu.memory[addr]);
        }

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    //
    // logic
    //
    // LOGIC - AND
    pub fn and(cpu: &mut Cpu, mode: AddressingMode) {
        fn _and(cpu: &mut Cpu, val: u8) {
            cpu.a &= val;
            cpu.update_zero(cpu.a == 0);
            cpu.update_negative(cpu.a & 0x80 != 0);
        }

        let addr = match mode {
            AddressingMode::Immediate => cpu._immediate(), // AND #$nn
            AddressingMode::Absolute => cpu._absolute(), // AND $nnnn
            AddressingMode::AbsoluteX => cpu._absolute_x(), // AND $nnnn,X
            AddressingMode::AbsoluteY => cpu._absolute_y(), // AND $nnnn,Y
            AddressingMode::ZeroPage => cpu._zero_page(), // AND $nn
            AddressingMode::ZeroPageX => cpu._zero_page_x(), // AND $nn,X
            AddressingMode::ZeroPageXIndirect => cpu._zero_page_x_indirect(), // AND ($nn,X)
            AddressingMode::ZeroPageIndirectY => cpu._zero_page_indirect_y(), // AND ($nn),Y
            _ => unimplemented!("bad addressing mode for the AND instruction"),
        };

        _and(cpu, cpu.memory[addr]);
        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // LOGIC - BIT - Test Bits in Memory with Accumulator
    pub fn bit(cpu: &mut Cpu, mode: AddressingMode) {
        fn _bit(cpu: &mut Cpu, val: u8) {
            let r = cpu.a & val;
            cpu.update_zero(r == 0);
            cpu.update_overflow(r & 0x40 != 0);
            cpu.update_negative(r & 0x80 != 0);
        }

        let addr = match mode {
            AddressingMode::Absolute => cpu._absolute(), // BIT $nnnn
            AddressingMode::ZeroPage => cpu._zero_page(), // BIT $nn
            _ => unimplemented!("bad addressing mode for the BIT instruction"),
        };

        _bit(cpu, cpu.memory[addr]);
        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // LOGIC - EOR - Test Bits in Memory with Accumulator
    pub fn eor(cpu: &mut Cpu, mode: AddressingMode) {
        fn _eor(cpu: &mut Cpu, val: u8) {
            cpu.a ^= val;
            cpu.update_zero(cpu.a == 0);
            cpu.update_negative(cpu.a & 0x80 != 0);
        }

        let addr = match mode {
            AddressingMode::Immediate => cpu._immediate(), // EOR #$nn
            AddressingMode::Absolute => cpu._absolute(), // EOR $nnnn
            AddressingMode::AbsoluteX => cpu._absolute_x(), // EOR $nnnn,X
            AddressingMode::AbsoluteY => cpu._absolute_y(), // EOR $nnnn,Y
            AddressingMode::ZeroPage => cpu._zero_page(), // EOR $nn
            AddressingMode::ZeroPageX => cpu._zero_page_x(), // EOR $nn,X
            AddressingMode::ZeroPageXIndirect => cpu._zero_page_x_indirect(), // EOR ($nn,X)
            AddressingMode::ZeroPageIndirectY => cpu._zero_page_indirect_y(), // EOR ($nn),Y
            _ => unimplemented!("bad addressing mode for the EOR instruction"),
        };

        _eor(cpu, cpu.memory[addr]);
        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // LOGIC - ORA - "Exclusive OR" Memory with Accumulator
    pub fn ora(cpu: &mut Cpu, mode: AddressingMode) {
        fn _ora(cpu: &mut Cpu, val: u8) {
            cpu.a |= val;
            cpu.update_zero(cpu.a == 0);
            cpu.update_negative(cpu.a & 0x80 != 0);
        }

        let addr = match mode {
            AddressingMode::Immediate => cpu._immediate(), // ORA #$nn
            AddressingMode::Absolute => cpu._absolute(), // ORA $nnnn
            AddressingMode::AbsoluteX => cpu._absolute_x(), // ORA $nnnn,X
            AddressingMode::AbsoluteY => cpu._absolute_y(), // ORA $nnnn,Y
            AddressingMode::ZeroPage => cpu._zero_page(), // ORA $nn
            AddressingMode::ZeroPageX => cpu._zero_page_x(), // ORA $nn,X
            AddressingMode::ZeroPageXIndirect => cpu._zero_page_x_indirect(), // ORA ($nn,X)
            AddressingMode::ZeroPageIndirectY => cpu._zero_page_indirect_y(), // ORA ($nn),Y
            _ => unimplemented!("bad addressing mode for the ORA instruction"),
        };

        _ora(cpu, cpu.memory[addr]);
        cpu.pc += addressing_mode_pc_advance(mode);
    }

    //
    // arith
    //
    // ARITH - ADC - Add Memory to Accumulator with Carry
    pub fn adc(cpu: &mut Cpu, mode: AddressingMode) {
        if cpu.is_decimal() {
            unimplemented!();
        }

        let addr = match mode {
            AddressingMode::Immediate => cpu._immediate(), // ADC #$nn
            AddressingMode::Absolute => cpu._absolute(), // ADC $nnnn
            AddressingMode::AbsoluteX => cpu._absolute_x(), // ADC $nnnn,X
            AddressingMode::AbsoluteY => cpu._absolute_y(), // ADC $nnnn,Y
            AddressingMode::ZeroPage => cpu._zero_page(), // ADC $nn
            AddressingMode::ZeroPageX => cpu._zero_page_x(), // ADC $nn,X
            AddressingMode::ZeroPageXIndirect => cpu._zero_page_x_indirect(), // ADC ($nn,X)
            AddressingMode::ZeroPageIndirectY => cpu._zero_page_indirect_y(), // ADC ($nn),Y
            _ => unimplemented!("bad addressing mode for the ADC instruction"),
        };


        let c: u16 = if cpu.is_carry() {
            1
        } else {
            0
        };
        let v = cpu.memory[addr];
        let r: u16 = cpu.a as u16 + v as u16 + c;
        cpu.a = r as u8;
        // TODO: Add overflow flag update
        cpu.update_negative(r & 0x80 != 0);
        cpu.update_zero(r & 0xff == 0);
        cpu.update_carry(r & 0x0100 != 0);

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // ARITH - CMP - Subtract Memory from Accumulator with Borrow
    pub fn cmp(cpu: &mut Cpu, mode: AddressingMode) {
        let addr = match mode {
            AddressingMode::Immediate => cpu._immediate(), // CMP #$nn
            AddressingMode::Absolute => cpu._absolute(), // CMP $nnnn
            AddressingMode::AbsoluteX => cpu._absolute_x(), // CMP $nnnn,X
            AddressingMode::AbsoluteY => cpu._absolute_y(), // CMP $nnnn,Y
            AddressingMode::ZeroPage => cpu._zero_page(), // CMP $nn
            AddressingMode::ZeroPageX => cpu._zero_page_x(), // CMP $nn,X
            AddressingMode::ZeroPageXIndirect => cpu._zero_page_x_indirect(), // CMP ($nn,X)
            AddressingMode::ZeroPageIndirectY => cpu._zero_page_indirect_y(), // CMP ($nn),Y
            _ => unimplemented!("bad addressing mode for the CMP instruction"),
        };

        let v = cpu.memory[addr];
        let r: u16 = (0x0100 + cpu.a as u16) - v as u16;
        cpu.update_negative(r & 0x80 != 0);
        cpu.update_zero(r & 0xff == 0);
        cpu.update_carry(r & 0x0100 != 0);

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // ARITH - CPX - Compare Index Register X To Memory
    pub fn cpx(cpu: &mut Cpu, mode: AddressingMode) {
        let addr = match mode {
            AddressingMode::Immediate => cpu._immediate(), // CPX #$nn
            AddressingMode::Absolute => cpu._absolute(), // CPX $nnnn
            AddressingMode::ZeroPage => cpu._zero_page(), // CPX $nn
            _ => unimplemented!("bad addressing mode for the CPX instruction"),
        };

        let v = cpu.memory[addr];
        let r: u16 = (0x0100 + cpu.x as u16) - v as u16;
        cpu.update_negative(r & 0x80 != 0);
        cpu.update_zero(r & 0xff == 0);
        cpu.update_carry(r & 0x0100 != 0);

        cpu.pc += addressing_mode_pc_advance(mode);
        // FIXME: 0x30 - 0x40 ? -0x10 ? should the N_Negative flags be set or not?
    }

    // ARITH - CPY - Compare Index Register Y To Memory
    pub fn cpy(cpu: &mut Cpu, mode: AddressingMode) {
        let addr = match mode {
            AddressingMode::Immediate => cpu._immediate(), // CPY #$nn
            AddressingMode::Absolute => cpu._absolute(), // CPY $nnnn
            AddressingMode::ZeroPage => cpu._zero_page(), // CPY $nn
            _ => unimplemented!("bad addressing mode for the CPY instruction"),
        };

        let v = cpu.memory[addr];
        let r: u16 = (0x0100 + cpu.y as u16) - v as u16;
        cpu.update_negative(r & 0x80 != 0);
        cpu.update_zero(r & 0xff == 0);
        cpu.update_carry(r & 0x0100 != 0);

        cpu.pc += addressing_mode_pc_advance(mode);
        // FIXME: 0x30 - 0x40 ? -0x10 ? should the N_Negative flags be set or not?
    }

    // ARITH - SBC - Add Memory to Accumulator with Carry
    pub fn sbc(cpu: &mut Cpu, mode: AddressingMode) {
        if cpu.is_decimal() {
            unimplemented!();
        }

        let addr = match mode {
            AddressingMode::Immediate => cpu._immediate(), // SBC #$nn
            AddressingMode::Absolute => cpu._absolute(), // SBC $nnnn
            AddressingMode::AbsoluteX => cpu._absolute_x(), // SBC $nnnn,X
            AddressingMode::AbsoluteY => cpu._absolute_y(), // SBC $nnnn,Y
            AddressingMode::ZeroPage => cpu._zero_page(), // SBC $nn
            AddressingMode::ZeroPageX => cpu._zero_page_x(), // SBC $nn,X
            AddressingMode::ZeroPageXIndirect => cpu._zero_page_x_indirect(), // SBC ($nn,X)
            AddressingMode::ZeroPageIndirectY => cpu._zero_page_indirect_y(), // SBC ($nn),Y
            _ => unimplemented!("bad addressing mode for the SBC instruction"),
        };

        let c: u16 = if cpu.is_carry() {
            1
        } else {
            0
        };
        let v = cpu.memory[addr];
        let r: u16 = 0x0100 + cpu.a as u16 - v as u16 - c;
        cpu.a = r as u8;

        // TODO: Add overflow flag update
        cpu.update_negative(r & 0x80 != 0);
        cpu.update_zero(r & 0xff == 0);
        cpu.update_carry(r & 0x0100 == 0);

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    //
    // inc
    //
    // INCREMENT - DEC
    pub fn dec(cpu: &mut Cpu, mode: AddressingMode) {
        let addr = match mode {
            AddressingMode::Absolute => cpu._absolute(),
            AddressingMode::AbsoluteX => cpu._absolute_x(),
            AddressingMode::ZeroPage => cpu._zero_page(),
            AddressingMode::ZeroPageX => cpu._zero_page_x(),
            _ => unimplemented!("bad addressing mode for the DEC instruction"),
        };

        let mut v = cpu.memory[addr];
        v = v.wrapping_sub(1);
        cpu.memory[addr] = v;

        cpu.update_negative(v & 0x80 != 0);
        cpu.update_zero(v == 0);

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // INCREMENT - DEX - Decrement Index Register X By One
    pub fn dex(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied, "bad addressing mode for the DEX instruction");
        cpu.x = cpu.x.wrapping_sub(1);
        cpu.update_negative(cpu.x & 0x80 != 0);
        cpu.update_zero(cpu.x & 0xff == 0);
        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // INCREMENT - DEY - Decrement Index Register Y By One
    pub fn dey(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied, "bad addressing mode for the DEY instruction");
        cpu.y = cpu.y.wrapping_sub(1);
        cpu.update_negative(cpu.y & 0x80 != 0);
        cpu.update_zero(cpu.y & 0xff == 0);
        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // INCREMENT - INC - Increment Memory By One
    pub fn inc(cpu: &mut Cpu, mode: AddressingMode) {
        let addr = match mode {
            AddressingMode::Absolute => cpu._absolute(),
            AddressingMode::AbsoluteX => cpu._absolute_x(),
            AddressingMode::ZeroPage => cpu._zero_page(),
            AddressingMode::ZeroPageX => cpu._zero_page_x(),
            _ => unimplemented!("bad addressing mode for the INC instruction"),
        };

        let mut v = cpu.memory[addr];
        v = v.wrapping_add(1);
        cpu.memory[addr] = v;

        cpu.update_negative(v & 0x80 != 0);
        cpu.update_zero(v == 0);

        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // INCREMENT - INX - Increment Index Register X By One
    pub fn inx(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied, "bad addressing mode for the DEX instruction");
        cpu.x = cpu.x.wrapping_add(1);
        cpu.update_negative(cpu.x & 0x80 != 0);
        cpu.update_zero(cpu.x & 0xff == 0);
        cpu.pc += addressing_mode_pc_advance(mode);
    }

    // INCREMENT - INY - Increment Index Register Y By One
    pub fn iny(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied, "bad addressing mode for the INY instruction");
        cpu.y = cpu.y.wrapping_add(1);
        cpu.update_negative(cpu.y & 0x80 != 0);
        cpu.update_zero(cpu.y & 0xff == 0);
        cpu.pc += addressing_mode_pc_advance(mode);
    }

    //
    // ctrl
    //
    pub fn brk(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied, "bad addressing mode for the BRK instruction");
        cpu.pc += addressing_mode_pc_advance(mode);
        unimplemented!();
    }

    pub fn jmp(cpu: &mut Cpu, mode: AddressingMode) {
        let addr = match mode {
            AddressingMode::Absolute => cpu._absolute(),
            AddressingMode::AbsoluteIndirect => cpu._absolute_indirect(),
            _ => panic!("bad addressing mode for the JMP instruction"),
        };

         // DO NOT DO THIS !!! // cpu.pc += addressing_mode_pc_advance(mode);
        cpu.pc = addr as u16;
    }

    pub fn jsr(cpu: &mut Cpu, mode: AddressingMode) {
        // TODO: functions for working with stack
        assert!(mode == AddressingMode::Absolute, "bad addressing mode for the JSR instruction");

        // store program counter on stack
        let pc = cpu.pc + 1; // +1 -> the last byte of this 3byte instruction. TODO: pay attention
        cpu.memory[0x0100 + cpu.s as usize] = (pc >> 8) as u8;
        cpu.s -= 1;
        cpu.memory[0x0100 + cpu.s as usize] = pc as u8;
        cpu.s -= 1;

        // load new program counter
        cpu.pc = cpu._absolute() as u16;
    }

    pub fn rti(cpu: &mut Cpu, mode: AddressingMode) {
        // TODO: add function to work with the stack
        assert!(mode == AddressingMode::Implied, "bad addressing mode for the RTI instruction");

        // restore status flags
        cpu.p = cpu.memory[0x0100 + cpu.s as usize];
        cpu.s += 1;

        // restore pc: low byte
        cpu.pc = cpu.memory[0x0100 + cpu.s as usize] as u16;
        cpu.s += 1;
        // restore pc: high byte
        cpu.pc |= (cpu.memory[0x0100 + cpu.s as usize] as u16) << 8;
        cpu.s += 1;

        // here += 1 should be used instead of addressing_mode_pc_advance
        cpu.pc += 1;
    }

    pub fn rts(cpu: &mut Cpu, mode: AddressingMode) {
        // TODO: add function to work with the stack
        assert!(mode == AddressingMode::Implied, "bad addressing mode for the RTS instruction");

        // low byte
        cpu.s += 1;
        cpu.pc = cpu.memory[0x0100 + cpu.s as usize] as u16;
        // high byte
        cpu.s += 1;
        cpu.pc |= (cpu.memory[0x0100 + cpu.s as usize] as u16) << 8;

        // here += 1 should be used instead of addressing_mode_pc_advance
        cpu.pc += 1;
    }

    //
    // bra
    //
    // BRANCH - BCC - Branch Carry Clear
    pub fn bcc(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Relative, "bad addressing mode for the BCC instruction");
        if !cpu.is_carry() {
            cpu.set_pc_to_current_addr_in_memory();
        } else {
            cpu.pc += 2;
        }
    }

    // BRANCH - BCS - Branch Carry Set
    pub fn bcs(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Relative, "bad addressing mode for the BCS instruction");
        if cpu.is_carry() {
            cpu.set_pc_to_current_addr_in_memory();
        } else {
            cpu.pc += 2;
        }
    }

    // BRANCH - BEQ - Branch Equal
    pub fn beq(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Relative, "bad addressing mode for the BEQ instruction");
        if cpu.is_zero() {
            cpu.set_pc_to_current_addr_in_memory();
        } else {
            cpu.pc += 2;
        }
    }

    // BRANCH - BMI - Branch MInus
    pub fn bmi(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Relative, "bad addressing mode for the BMI instruction");
        if cpu.is_negative() {
            cpu.set_pc_to_current_addr_in_memory();
        } else {
            cpu.pc += 2;
        }
    }

    // BRANCH - BNE - Branch Not Equal
    pub fn bne(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Relative, "bad addressing mode for the BNE instruction");
        if !cpu.is_zero() {
            cpu.set_pc_to_current_addr_in_memory();
        } else {
            cpu.pc += 2;
        }
    }

    // BRANCH - BPL - Branch PLus
    pub fn bpl(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Relative, "bad addressing mode for the BPL instruction");
        if !cpu.is_negative() {
            cpu.set_pc_to_current_addr_in_memory();
        } else {
            cpu.pc += 2;
        }
    }

    // BRANCH - BVC - Branch oVerflow Clear
    pub fn bvc(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Relative, "bad addressing mode for the BVC instruction");
        if !cpu.is_overflow() {
            cpu.set_pc_to_current_addr_in_memory();
        } else {
            cpu.pc += 2;
        }
    }

    // BRANCH - BVS - Branch oVerflow Set
    pub fn bvs(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Relative, "bad addressing mode for the BVS instruction");
        if cpu.is_overflow() {
            cpu.set_pc_to_current_addr_in_memory();
        } else {
            cpu.pc += 2;
        }
    }

    //
    // flags
    //
    // FLAGS - CLC - Clear Carry Flag
    pub fn clc(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied);
        cpu.update_carry(false);
        cpu.pc += addressing_mode_pc_advance(mode); // SHOULD BE 0
    }

    // FLAGS - CLD - Clear Decimal Mode
    pub fn cld(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied);
        cpu.update_decimal(false);
        cpu.pc += addressing_mode_pc_advance(mode); // SHOULD BE 0
    }

    // FLAGS - CLI - Clear Interrupt Disable
    pub fn cli(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied);
        cpu.update_interrupt_disable(false);
        cpu.pc += addressing_mode_pc_advance(mode); // SHOULD BE 0
    }

    // FLAGS - CLV - Clear Overflow Flag
    pub fn clv(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied);
        cpu.update_overflow(false);
        cpu.pc += addressing_mode_pc_advance(mode); // SHOULD BE 0
    }

    // FLAGS - SEC - Set Carry Flag
    pub fn sec(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied);
        cpu.update_carry(true);
        cpu.pc += addressing_mode_pc_advance(mode); // SHOULD BE 0
    }

    // FLAGS - SED - Set Decimal Mode
    pub fn sed(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied);
        cpu.update_decimal(true);
        cpu.pc += addressing_mode_pc_advance(mode); // SHOULD BE 0
    }

    // FLAGS - SEI - Set Interrupt Disable
    pub fn sei(cpu: &mut Cpu, mode: AddressingMode) {
        assert!(mode == AddressingMode::Implied);
        cpu.update_interrupt_disable(true);
        cpu.pc += addressing_mode_pc_advance(mode); // SHOULD BE 0
    }

    //
    // nop
    //
    pub fn nop(_cpu: &mut Cpu, _mode: AddressingMode) {
        // ;
    }
}

// FIXME:
#[allow(dead_code)]
impl Cpu {
    pub fn new() -> Cpu {
        Cpu {
            a: 0,
            x: 0,
            y: 0,
            pc: 0,
            s: 0xff,
            p: 0,

            memory: [0; MEM_SZ],
        }
    }

    fn _dump_memory(&self) {
        fn print_line(l: &[u8]) {
            for i in 0..l.len() {
                let v = l[i];
                if v != 0 {
                    print!("\x1b[31;1m");
                }
                print!(" {:02x}", v);
                if v != 0 {
                    print!("\x1b[0m");
                }
                if (i+1) % 8 == 0 {
                    print!("   ");
                }
            }
        }

        const LINE_SIZE: usize = 16;
        let mut is_all_zero: bool = false;
        let mut printed: bool = false;

        let mut addr = 0usize;
        while addr < MEM_SZ {
            if !is_all_zero && &self.memory[addr .. addr + LINE_SIZE] == &[0u8; LINE_SIZE] {
                is_all_zero = true;
            } else if is_all_zero &&
                !printed &&
                &self.memory[addr .. addr + LINE_SIZE] == &[0u8; LINE_SIZE]
            {
                println!("        ...");
                printed = true;
                addr += LINE_SIZE;
                continue;
            } else if &self.memory[addr .. addr + LINE_SIZE] != &[0u8; LINE_SIZE] {
                is_all_zero = false;
                printed = false;
            } else if printed {
                addr += LINE_SIZE;
                continue;
            }

            print!("0x{addr:04x}:");
            print_line(&self.memory[addr .. addr + LINE_SIZE]);
            println!();
            addr += LINE_SIZE;
        }
    }

    //
    // FLAGS
    //

    fn is_carry(&self) -> bool {
        self.p & Flags::C_Carry != 0
    }

    fn is_decimal(&self) -> bool {
        self.p & Flags::D_Decimal != 0
    }

    fn is_interrupt_disabled(&self) -> bool {
        self.p & Flags::I_InterruptDisable != 0
    }

    fn is_negative(&self) -> bool {
        self.p & Flags::N_Negative != 0
    }

    fn is_overflow(&self) -> bool {
        self.p & Flags::V_Overflow != 0
    }

    fn is_zero(&self) -> bool {
        self.p & Flags::Z_Zero != 0
    }

    fn update_negative(&mut self, flag: bool) {
        if flag {
            self.p |= Flags::N_Negative;
        } else {
            self.p &= !Flags::N_Negative;
        }
    }

    fn update_zero(&mut self, flag: bool) {
        if flag {
            self.p |= Flags::Z_Zero;
        } else {
            self.p &= !Flags::Z_Zero;
        }
    }

    fn update_overflow(&mut self, flag: bool) {
        if flag {
            self.p |= Flags::V_Overflow;
        } else {
            self.p &= !Flags::V_Overflow;
        }
    }

    fn update_carry(&mut self, flag: bool) {
        if flag {
            self.p |= Flags::C_Carry;
        } else {
            self.p &= !Flags::C_Carry;
        }
    }

    fn update_decimal(&mut self, flag: bool) {
        if flag {
            self.p |= Flags::D_Decimal;
        } else {
            self.p &= !Flags::D_Decimal;
        }
    }

    fn update_interrupt_disable(&mut self, flag: bool) {
        if flag {
            self.p |= Flags::I_InterruptDisable;
        } else {
            self.p &= !Flags::I_InterruptDisable;
        }
    }

    fn set_pc_to_current_addr_in_memory(&mut self) {
        self.pc = self._absolute() as u16;
    }



    //
    // Addressing
    //
    fn _immediate(&mut self) -> usize {
        self.pc as usize
    }

    fn _absolute(&mut self) -> usize {
        let mut addr = self.memory[self.pc as usize] as usize;
        addr |= (self.memory[self.pc as usize + 1] as usize) << 8;
        addr
    }

    fn _absolute_indirect(&mut self) -> usize {
        let addr = self._absolute();
        let mut addr2 = self.memory[addr] as usize;
        addr2 |= (self.memory[addr+1] as usize) << 8;
        addr2
    }

    fn _absolute_x(&mut self) -> usize {
        self._absolute() + self.x as usize
    }

    fn _absolute_y(&mut self) -> usize {
        self._absolute() + self.y as usize
    }

    fn _zero_page(&mut self) -> usize {
        self.memory[self.pc as usize] as usize
    }

    fn _zero_page_x(&mut self) -> usize {
        (self._zero_page() + self.x as usize) & 0xff
    }

    fn _zero_page_y(&mut self) -> usize {
        (self._zero_page() + self.y as usize) & 0xff
    }

    fn _zero_page_x_indirect(&mut self) -> usize {
        let addr = self._zero_page_x();
        let mut addr2 = self.memory[addr] as usize;
        addr2 += (self.memory[(addr + 1) & 0xff] as usize) << 8;
        addr2
    }

    fn _zero_page_indirect_y(&mut self) -> usize {
        let addr = self._zero_page();
        let mut addr2 = self.memory[addr] as usize; // lo
        addr2 += (self.memory[(addr+1) & 0xff] as usize) << 8; // hi
        addr2 += self.y as usize;
        addr2
    }

    fn reset(&mut self) {
        self.a = 0;
        self.x = 0;
        self.y = 0;
        self.pc = 0;
        self.s = 0xff;
        self.p = 0;
    }

    fn step(&mut self) {
        let opcode = self.memory[self.pc as usize];
        self.pc += 1;
        match opcode {
            //
            // LOAD - LDA
            //
            opcodes::LDA_A9 => instructions::lda(self, AddressingMode::Immediate), // LDA #$nn
            opcodes::LDA_AD => instructions::lda(self, AddressingMode::Absolute), // LDA $nnnn
            opcodes::LDA_BD => instructions::lda(self, AddressingMode::AbsoluteX), // LDA $nnnn,x
            opcodes::LDA_B9 => instructions::lda(self, AddressingMode::AbsoluteY), // LDA $nnnn,y
            opcodes::LDA_A5 => instructions::lda(self, AddressingMode::ZeroPage), // LDA $nn
            opcodes::LDA_B5 => instructions::lda(self, AddressingMode::ZeroPageX), // LDA $nn,X
            opcodes::LDA_A1 => instructions::lda(self, AddressingMode::ZeroPageXIndirect), // LDA ($nn,X)
            opcodes::LDA_B1 => instructions::lda(self, AddressingMode::ZeroPageIndirectY), // LDA ($nn),Y

            //
            // LOAD - LDX
            //
            opcodes::LDX_A2 => instructions::ldx(self, AddressingMode::Immediate), // LDX #$nn
            opcodes::LDX_AE => instructions::ldx(self, AddressingMode::Absolute), // LDX $nnnn
            opcodes::LDX_BE => instructions::ldx(self, AddressingMode::AbsoluteY), // LDX $nnnn,Y
            opcodes::LDX_A6 => instructions::ldx(self, AddressingMode::ZeroPage), // LDX $nn
            opcodes::LDX_B6 => instructions::ldx(self, AddressingMode::ZeroPageY), // LDX $nn,Y

            //
            // LOAD - LDY
            //
            opcodes::LDY_A0 => instructions::ldy(self, AddressingMode::Immediate), // LDY #$nn
            opcodes::LDY_AC => instructions::ldy(self, AddressingMode::Absolute), // LDY $nnnn
            opcodes::LDY_BC => instructions::ldy(self, AddressingMode::AbsoluteX), // LDY $nnnn,X
            opcodes::LDY_A4 => instructions::ldy(self, AddressingMode::ZeroPage), // LDY $nn
            opcodes::LDY_B4 => instructions::ldy(self, AddressingMode::ZeroPageX), // LDY $nn,X

            //
            // LOAD - STA
            //
            opcodes::STA_8D => instructions::sta(self, AddressingMode::Absolute), // STA $nnnn
            opcodes::STA_9D => instructions::sta(self, AddressingMode::AbsoluteX), // STA $nnnn,X
            opcodes::STA_99 => instructions::sta(self, AddressingMode::AbsoluteY), // STA $nnnn,Y
            opcodes::STA_85 => instructions::sta(self, AddressingMode::ZeroPage), // STA $nn
            opcodes::STA_95 => instructions::sta(self, AddressingMode::ZeroPageX), // STA $nn,X
            opcodes::STA_81 => instructions::sta(self, AddressingMode::ZeroPageXIndirect), // STA ($nn,X)
            opcodes::STA_91 => instructions::sta(self, AddressingMode::ZeroPageIndirectY), // STA ($nn),Y

            //
            // LOAD - STX
            //
            opcodes::STX_8E => instructions::stx(self, AddressingMode::Absolute), // STX $nnnn
            opcodes::STX_86 => instructions::stx(self, AddressingMode::ZeroPage), // STX $nn
            opcodes::STX_96 => instructions::stx(self, AddressingMode::ZeroPageY), // STX $nn,Y

            //
            // LOAD - STY
            //
            opcodes::STY_8C => instructions::sty(self, AddressingMode::Absolute), // STY $nnnn
            opcodes::STY_84 => instructions::sty(self, AddressingMode::ZeroPage), // STY $nn
            opcodes::STY_94 => instructions::sty(self, AddressingMode::ZeroPageX), // STY $nn,X

            //
            // INCREMENT - INC, INX, INY
            //
            opcodes::INC_EE => instructions::inc(self, AddressingMode::Absolute), // INC $nnnn
            opcodes::INC_FE => instructions::inc(self, AddressingMode::AbsoluteX), // INC $nnnn,X
            opcodes::INC_E6 => instructions::inc(self, AddressingMode::ZeroPage), // INC $nn
            opcodes::INC_F6 => instructions::inc(self, AddressingMode::ZeroPageX), // INC $nn,X

            opcodes::INX_E8 => instructions::inx(self, AddressingMode::Implied),
            opcodes::INY_C8 => instructions::iny(self, AddressingMode::Implied),

            //
            // ARITH - ADC
            //
            opcodes::ADC_69 => instructions::adc(self, AddressingMode::Immediate), // ADC #$nn
            opcodes::ADC_6D => instructions::adc(self, AddressingMode::Absolute), // ADC $nnnn
            opcodes::ADC_7D => instructions::adc(self, AddressingMode::AbsoluteX), // ADC $nnnn,X
            opcodes::ADC_79 => instructions::adc(self, AddressingMode::AbsoluteY), // ADC $nnnn,Y
            opcodes::ADC_65 => instructions::adc(self, AddressingMode::ZeroPage), // ADC $nn
            opcodes::ADC_75 => instructions::adc(self, AddressingMode::ZeroPageX), // ADC $nn,X
            opcodes::ADC_61 => instructions::adc(self, AddressingMode::ZeroPageXIndirect), // ADC ($nn,X)
            opcodes::ADC_71 => instructions::adc(self, AddressingMode::ZeroPageIndirectY), // ADC ($nn),Y

            //
            // ARITH - SBC
            //
            opcodes::SBC_E9 => instructions::sbc(self, AddressingMode::Immediate), // SBC #$nn
            opcodes::SBC_ED => instructions::sbc(self, AddressingMode::Absolute), // SBC $nnnn
            opcodes::SBC_FD => instructions::sbc(self, AddressingMode::AbsoluteX), // SBC $nnnn,X
            opcodes::SBC_F9 => instructions::sbc(self, AddressingMode::AbsoluteY), // SBC $nnnn,Y
            opcodes::SBC_E5 => instructions::sbc(self, AddressingMode::ZeroPage), // SBC $nn
            opcodes::SBC_F5 => instructions::sbc(self, AddressingMode::ZeroPageX), // SBC $nn,X
            opcodes::SBC_E1 => instructions::sbc(self, AddressingMode::ZeroPageXIndirect), // SBC ($nn,X)
            opcodes::SBC_F1 => instructions::sbc(self, AddressingMode::ZeroPageIndirectY), // SBC ($nn),Y

            //
            // ARITH - CMP
            //
            opcodes::CMP_C9 => instructions::cmp(self, AddressingMode::Immediate), // CMP #$nn
            opcodes::CMP_CD => instructions::cmp(self, AddressingMode::Absolute), // CMP $nnnn
            opcodes::CMP_DD => instructions::cmp(self, AddressingMode::AbsoluteX), // CMP $nnnn,X
            opcodes::CMP_D9 => instructions::cmp(self, AddressingMode::AbsoluteY), // CMP $nnnn,Y
            opcodes::CMP_C5 => instructions::cmp(self, AddressingMode::ZeroPage), // CMP $nn
            opcodes::CMP_D5 => instructions::cmp(self, AddressingMode::ZeroPageX), // CMP $nn,X
            opcodes::CMP_C1 => instructions::cmp(self, AddressingMode::ZeroPageXIndirect), // CMP ($nn,X)
            opcodes::CMP_D1 => instructions::cmp(self, AddressingMode::ZeroPageIndirectY), // CMP ($nn),Y

            //
            // ARITH - CPX
            //
            // FIXME: 0x30 - 0x40 ? -0x10 ? should the N_Negative flags be set or not?
            opcodes::CPX_E0 => instructions::cpx(self, AddressingMode::Immediate), // CPX #$nn
            opcodes::CPX_EC => instructions::cpx(self, AddressingMode::Absolute), // CPX $nnnn
            opcodes::CPX_E4 => instructions::cpx(self, AddressingMode::ZeroPage), // CPX $nn

            //
            // ARITH - CPY
            //
            opcodes::CPY_C0 => instructions::cpy(self, AddressingMode::Immediate), // CPY #$nn
            opcodes::CPY_CC => instructions::cpy(self, AddressingMode::Absolute), // CPY $nnnn
            opcodes::CPY_C4 => instructions::cpy(self, AddressingMode::ZeroPage), // CPY $nn

            //
            // INCREMENT - DEC, DEX, DEY
            //
            opcodes::DEC_CE => instructions::dec(self, AddressingMode::Absolute), // DEC $nnnn
            opcodes::DEC_DE => instructions::dec(self, AddressingMode::AbsoluteX), // DEC $nnnn,X
            opcodes::DEC_C6 => instructions::dec(self, AddressingMode::ZeroPage), // DEC $nn
            opcodes::DEC_D6 => instructions::dec(self, AddressingMode::ZeroPageX), // DEC $nn,X

            opcodes::DEX_CA => instructions::dex(self, AddressingMode::Implied),
            opcodes::DEY_88 => instructions::dey(self, AddressingMode::Implied),

            //
            // FLAGS - CLC, CLD, CLI, CLV, SEC, SED, SEI
            //
            opcodes::CLC_18 => instructions::clc(self, AddressingMode::Implied),
            opcodes::CLD_D8 => instructions::cld(self, AddressingMode::Implied),
            opcodes::CLI_58 => instructions::cli(self, AddressingMode::Implied),
            opcodes::CLV_B8 => instructions::clv(self, AddressingMode::Implied),
            opcodes::SEC_38 => instructions::sec(self, AddressingMode::Implied),
            opcodes::SED_F8 => instructions::sed(self, AddressingMode::Implied),
            opcodes::SEI_78 => instructions::sei(self, AddressingMode::Implied),

            //
            // TRANSFER - TAX, TAY, TSX, TXA, TXS, TYA
            //
            opcodes::TAX_AA => instructions::tax(self, AddressingMode::Implied),
            opcodes::TAY_A8 => instructions::tay(self, AddressingMode::Implied),
            opcodes::TSX_BA => instructions::tsx(self, AddressingMode::Implied),
            opcodes::TXA_8A => instructions::txa(self, AddressingMode::Implied),
            opcodes::TXS_9A => instructions::txs(self, AddressingMode::Implied),
            opcodes::TYA_98 => instructions::tya(self, AddressingMode::Implied),

            //
            // STACK - PHA, PHP, PLA, PLP
            //
            opcodes::PHA_48 => instructions::pha(self, AddressingMode::Implied),
            opcodes::PHP_08 => instructions::php(self, AddressingMode::Implied),
            opcodes::PLA_68 => instructions::pla(self, AddressingMode::Implied),
            opcodes::PLP_28 => instructions::plp(self, AddressingMode::Implied),

            //
            // CONTROL
            //
            opcodes::BRK_00 => instructions::brk(self, AddressingMode::Implied),
            opcodes::JMP_4C => instructions::jmp(self, AddressingMode::Absolute), // JMP $nnnn
            opcodes::JMP_6C => instructions::jmp(self, AddressingMode::AbsoluteIndirect), // JMP ($nnnn)
            opcodes::JSR_20 => instructions::jsr(self, AddressingMode::Absolute), // JSR $nnnn
            opcodes::RTI_40 => instructions::rti(self, AddressingMode::Implied),
            opcodes::RTS_60 => instructions::rts(self, AddressingMode::Implied),

            //
            // BRANCH
            //
            opcodes::BCC_90 => instructions::bcc(self, AddressingMode::Relative), // BCC
            opcodes::BCS_B0 => instructions::bcs(self, AddressingMode::Relative), // BCS
            opcodes::BEQ_F0 => instructions::beq(self, AddressingMode::Relative), // BEQ
            opcodes::BMI_30 => instructions::bmi(self, AddressingMode::Relative), // BMI
            opcodes::BNE_D0 => instructions::bne(self, AddressingMode::Relative), // BNE
            opcodes::BPL_10 => instructions::bpl(self, AddressingMode::Relative), // BPL
            opcodes::BVC_50 => instructions::bvc(self, AddressingMode::Relative), // BVC
            opcodes::BVS_70 => instructions::bvs(self, AddressingMode::Relative), // BVS

            //
            // SHIFT - ASL - Arithmetic Shift Left
            //
            opcodes::ASL_0A => instructions::asl(self, AddressingMode::Accumulator), // ASL A
            opcodes::ASL_0E => instructions::asl(self, AddressingMode::Absolute), // ASL $nnnn
            opcodes::ASL_1E => instructions::asl(self, AddressingMode::AbsoluteX), // ASL $nnnn,X
            opcodes::ASL_06 => instructions::asl(self, AddressingMode::ZeroPage), // ASL $nn
            opcodes::ASL_16 => instructions::asl(self, AddressingMode::ZeroPageX), // ASL $nn,X

            //
            // SHIFT - LSR - Logic Shift Right
            //
            opcodes::LSR_4A => instructions::lsr(self, AddressingMode::Accumulator), // LSR A
            opcodes::LSR_4E => instructions::lsr(self, AddressingMode::Absolute), // LSR $nnnn
            opcodes::LSR_5E => instructions::lsr(self, AddressingMode::AbsoluteX), // LSR $nnnn,X
            opcodes::LSR_46 => instructions::lsr(self, AddressingMode::ZeroPage), // LSR $nn
            opcodes::LSR_56 => instructions::lsr(self, AddressingMode::ZeroPageX), // LSR $nn,X

            //
            // SHIFT - ROL - Rotate Left
            //
            opcodes::ROL_2A => instructions::rol(self, AddressingMode::Accumulator), // ROL A
            opcodes::ROL_2E => instructions::rol(self, AddressingMode::Absolute), // ROL $nnnn
            opcodes::ROL_3E => instructions::rol(self, AddressingMode::AbsoluteX), // ROL $nnnn,X
            opcodes::ROL_26 => instructions::rol(self, AddressingMode::ZeroPage), // ROL $nn
            opcodes::ROL_36 => instructions::rol(self, AddressingMode::ZeroPageX), // ROL $nn,X

            //
            // SHIFT - ROR - Rotate Right
            //
            opcodes::ROR_6A => instructions::ror(self, AddressingMode::Accumulator), // ROR A
            opcodes::ROR_6E => instructions::ror(self, AddressingMode::Absolute), // ROR $nnnn
            opcodes::ROR_7E => instructions::ror(self, AddressingMode::AbsoluteX), // ROR $nnnn,X
            opcodes::ROR_66 => instructions::ror(self, AddressingMode::ZeroPage), // ROR $nn
            opcodes::ROR_76 => instructions::ror(self, AddressingMode::ZeroPageX), // ROR $nn,X

            //
            // LOGIC - AND
            //
            opcodes::AND_29 => instructions::and(self, AddressingMode::Immediate), // AND #$nn
            opcodes::AND_2D => instructions::and(self, AddressingMode::Absolute), // AND $nnnn
            opcodes::AND_3D => instructions::and(self, AddressingMode::AbsoluteX), // AND $nnnn,X
            opcodes::AND_39 => instructions::and(self, AddressingMode::AbsoluteY), // AND $nnnn,Y
            opcodes::AND_25 => instructions::and(self, AddressingMode::ZeroPage), // AND $nn
            opcodes::AND_35 => instructions::and(self, AddressingMode::ZeroPageX), // AND $nn,X
            opcodes::AND_21 => instructions::and(self, AddressingMode::ZeroPageXIndirect), // AND ($nn,X)
            opcodes::AND_31 => instructions::and(self, AddressingMode::ZeroPageIndirectY), // AND ($nn),Y

            //
            // BIT
            //
            opcodes::BIT_2C => instructions::bit(self, AddressingMode::Absolute), // BIT $nnnn
            opcodes::BIT_24 => instructions::bit(self, AddressingMode::ZeroPage), // BIT $nn

            //
            // EOR
            //
            opcodes::EOR_49 => instructions::eor(self, AddressingMode::Immediate), // EOR #$49
            opcodes::EOR_4D => instructions::eor(self, AddressingMode::Absolute), // EOR $nnnn
            opcodes::EOR_5D => instructions::eor(self, AddressingMode::AbsoluteX), // EOR $nnnn,X
            opcodes::EOR_59 => instructions::eor(self, AddressingMode::AbsoluteY), // EOR $nnnn,Y
            opcodes::EOR_45 => instructions::eor(self, AddressingMode::ZeroPage), // EOR $nn
            opcodes::EOR_55 => instructions::eor(self, AddressingMode::ZeroPageX), // EOR $nn,X
            opcodes::EOR_41 => instructions::eor(self, AddressingMode::ZeroPageXIndirect), // EOR ($nn,X)
            opcodes::EOR_51 => instructions::eor(self, AddressingMode::ZeroPageIndirectY), // EOR ($nn),Y

            //
            // ORA
            //
            opcodes::ORA_09 => instructions::ora(self, AddressingMode::Immediate), // ORA #$nn
            opcodes::ORA_0D => instructions::ora(self, AddressingMode::Absolute), // ORA $nnnn
            opcodes::ORA_1D => instructions::ora(self, AddressingMode::AbsoluteX), // ORA $nnnn,X
            opcodes::ORA_19 => instructions::ora(self, AddressingMode::AbsoluteY), // ORA $nnnn,Y
            opcodes::ORA_05 => instructions::ora(self, AddressingMode::ZeroPage), // ORA $nn
            opcodes::ORA_15 => instructions::ora(self, AddressingMode::ZeroPageX), // ORA $nn,X
            opcodes::ORA_01 => instructions::ora(self, AddressingMode::ZeroPageXIndirect), // ORA ($nn,X)
            opcodes::ORA_11 => instructions::ora(self, AddressingMode::ZeroPageIndirectY), // ORA ($nn),Y

            //
            // NOP
            //
            opcodes::NOP_EA => instructions::nop(self, AddressingMode::Implied),

            _ => unimplemented!("opcode: {:02x} is not implemented", opcode),
        }
    }

    pub fn run(&mut self) {
        loop {
            println!(": pc: 0x{:04x}/{}    opcode: {:02x}", self.pc, self.pc, self.memory[self.pc as usize]);
            if self.pc == 0x0606 {
                self._dump_memory();
            }
            self.step();
        }
    }

    pub fn patch_memory(&mut self, offset: usize, bytes: &[u8]) {
        for (idx, b) in bytes.iter().enumerate() {
            self.memory[offset + idx] = *b;
        }
    }

    pub fn update_pc(&mut self, new_pc: u16) {
        self.pc = new_pc;
    }
}

#[cfg(test)]
mod tests {
    use super::{opcodes::*, Flags::*, *};

    //
    // LOAD - LDA
    //
    #[test]
    fn test_lda_a9() { // LDA #$nn
        fn _t(mem: &[u8], exp_a: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            assert!(cpu.a == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_p);
        }

        let mem1 = &[LDA_A9, 0x55];
        let mem2 = &[LDA_A9, 0];    // LDA #0 // Z_Zero
        let mem3 = &[LDA_A9, 0x80]; // N_Negative
        let mem4 = &[LDA_A9, 0xAA];

        _t(mem1, 0x55, 0);
        _t(mem2, 0x00, Z_Zero);
        _t(mem3, 0x80, N_Negative);
        _t(mem4, 0xAA, N_Negative);
    }

    #[test]
    fn test_lda_ad() { // LDA $nnnn
        fn _t(mem: &[u8], addr: usize, exp_a: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.patch_memory(addr, &[exp_a]);
            assert!(cpu.a == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_p);
        }

        let mem = &[LDA_AD, 0x20, 0x40];
        _t(mem, 0x4020, 0x00, Z_Zero);
        _t(mem, 0x4020, 0x80, N_Negative);
        _t(mem, 0x4020, 0x55, 0);
        _t(mem, 0x4020, 0xAA, N_Negative);
    }

    #[test]
    fn test_lda_bd() { // LDA $nnnn,X
        fn _t(mem: &[u8], addr: usize, x: u8, exp_a: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.patch_memory(addr + x as usize, &[exp_a]);
            cpu.x = x;
            assert!(cpu.a == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_p);
        }

        let mem = &[LDA_BD, 0x00, 0x80];
        _t(mem, 0x8000, 0x15, 0x00, Z_Zero);
        _t(mem, 0x8000, 0x15, 0x80, N_Negative);
        _t(mem, 0x8000, 0x15, 0x55, 0);
        _t(mem, 0x8000, 0x15, 0xAA, N_Negative);
    }

    #[test]
    fn test_lda_b9() { // LDA $nnnn,Y
        fn _t(mem: &[u8], addr: usize, y: u8, exp_a: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.patch_memory(addr + y as usize, &[exp_a]);
            cpu.y = y;
            assert!(cpu.a == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_p);
        }

        let mem = &[LDA_B9, 0x00, 0x80];
        _t(mem, 0x8000, 0x15, 0x00, Z_Zero);
        _t(mem, 0x8000, 0x15, 0x80, N_Negative);
        _t(mem, 0x8000, 0x15, 0x55, 0);
        _t(mem, 0x8000, 0x15, 0xAA, N_Negative);
    }

    #[test]
    fn test_lda_a5() { // LDA $nn
        fn _t(mem: &[u8], addr: usize, exp_a: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.patch_memory(addr, &[exp_a]);
            assert!(cpu.a == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_p);
        }

        let mem = &[LDA_A5, 0x28];
        _t(mem, 0x0028, 0x00, Z_Zero);
        _t(mem, 0x0028, 0x80, N_Negative);
        _t(mem, 0x0028, 0x55, 0);
        _t(mem, 0x0028, 0xAA, N_Negative);
    }

    #[test]
    fn test_lda_b5() { // LDA $nn,X
        fn _t(mem: &[u8], x: u8, exp_a: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.x = x;
            assert!(cpu.a == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_p);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = LDA_B5;
        mem[1] = 0x28;

        mem[0x28 + 0x03] = 0x00;
        _t(&mem, 0x03, 0x00, Z_Zero);

        mem[0x28 + 0x03] = 0x80;
        _t(&mem, 0x03, 0x80, N_Negative);

        mem[0x28 + 0x03] = 0x55;
        _t(&mem, 0x03, 0x55, 0);

        mem[0x28 + 0x03] = 0xAA;
        _t(&mem, 0x03, 0xAA, N_Negative);
    }

    #[test]
    fn test_lda_a1() {
        fn _t(mem: &[u8], x: u8, exp_a: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.x = x;
            assert!(cpu.a == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_p);
        }

        //                0     1     2     3     4
        let mem1 = &[LDA_A1, 0x01, 0x04, 0x00, 0x00]; // LDA ($nn,X)
        let mem2 = &[LDA_A1, 0x01, 0x04, 0x00, 0x80]; // LDA ($nn,X)
        let mem3 = &[LDA_A1, 0x01, 0x04, 0x00, 0x55]; // LDA ($nn,X)
        let mem4 = &[LDA_A1, 0x01, 0x04, 0x00, 0xAA]; // LDA ($nn,X)

        //  mem     x exp_a  exp_p
        _t(mem1, 0x01, 0x00, Z_Zero);
        _t(mem2, 0x01, 0x80, N_Negative);
        _t(mem3, 0x01, 0x55, 0);
        _t(mem4, 0x01, 0xAA, N_Negative);
    }

    #[test]
    fn test_lda_b1() {
        fn _t(mem: &[u8], y: u8, exp_a: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.y = y;
            assert!(cpu.a == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_p);
        }

        // TODO: FIXME:
        // Add more test: 2 bytes in page zero points to memory not in page zero
        // TODO Expand this tests so that memory address is outside of page zero

        //               0     1     2     3     4
        let mem1 = &[LDA_B1, 0x02, 0x01, 0x00, 0x00]; // LDA ($nn),Y
        let mem2 = &[LDA_B1, 0x02, 0x03, 0x00, 0x80]; // LDA ($nn),Y
        let mem3 = &[LDA_B1, 0x02, 0x01, 0x00, 0x55]; // LDA ($nn),Y
        let mem4 = &[LDA_B1, 0x02, 0x03, 0x00, 0xAA]; // LDA ($nn),Y

        //  mem     y exp_a  exp_p
        _t(mem1, 0x03, 0x00, Z_Zero);
        _t(mem2, 0x01, 0x80, N_Negative);
        _t(mem3, 0x03, 0x55, 0);
        _t(mem4, 0x01, 0xAA, N_Negative);
    }

    //
    // LOAD - LDX
    //
    #[test]
    fn test_ldx_a2() { // LDX #$nn
        fn _t(mem: &[u8], exp_x: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            assert!(cpu.x == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.x == exp_x);
            assert!(cpu.p == exp_p);
        }

        let mem1 = &[LDX_A2, 0];
        let mem2 = &[LDX_A2, 0x55];
        let mem3 = &[LDX_A2, 0x80];
        let mem4 = &[LDX_A2, 0xAA];

        _t(mem1, 0, Z_Zero);
        _t(mem2, 0x55, 0);
        _t(mem3, 0x80, N_Negative);
        _t(mem4, 0xAA, N_Negative);
    }

    #[test]
    fn test_ldx_ae() { // LDX $nnnn
        fn _t(mem: &[u8], exp_x: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            assert!(cpu.x == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.x == exp_x);
            assert!(cpu.p == exp_p);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = LDX_AE;
        mem[1] = 0x20;
        mem[2] = 0x40;

        mem[0x4020] = 0;
        _t(&mem, 0x00, Z_Zero);

        mem[0x4020] = 0x80;
        _t(&mem, 0x80, N_Negative);

        mem[0x4020] = 0x55;
        _t(&mem, 0x55, 0);

        mem[0x4020] = 0xAA;
        _t(&mem, 0xAA, N_Negative);
    }

    #[test]
    fn test_ldx_be() { // LDX $nnnn,Y
        fn _t(mem: &[u8], y: u8, exp_x: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.y = y;
            assert!(cpu.x == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.x == exp_x);
            assert!(cpu.p == exp_p);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = LDX_BE;
        mem[1] = 0x60;
        mem[2] = 0x70;

        mem[0x7075] = 0;
        _t(&mem, 0x15, 0x00, Z_Zero);

        mem[0x7075] = 0x55;
        _t(&mem, 0x15, 0x55, 0);

        mem[0x7075] = 0x80;
        _t(&mem, 0x15, 0x80, N_Negative);

        mem[0x7075] = 0xAA;
        _t(&mem, 0x15, 0xAA, N_Negative);
    }

    #[test]
    fn test_ldx_a6() { // LDX $nn
        fn _t(mem: &[u8], exp_x: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            assert!(cpu.x == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.x == exp_x);
            assert!(cpu.p == exp_p);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = LDX_A6;
        mem[1] = 0x60;

        mem[0x0060] = 0;
        _t(&mem, 0x00, Z_Zero);

        mem[0x0060] = 0x55;
        _t(&mem, 0x55, 0);

        mem[0x0060] = 0x80;
        _t(&mem, 0x80, N_Negative);

        mem[0x0060] = 0xAA;
        _t(&mem, 0xAA, N_Negative);
    }

    #[test]
    fn test_ldx_b6() { // LDX $nn,Y
        fn _t(mem: &[u8], y: u8, exp_x: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.y = y;
            assert!(cpu.x == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.x == exp_x);
            assert!(cpu.p == exp_p);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = LDX_B6;
        mem[1] = 0x60;

        mem[0x0075] = 0;
        _t(&mem, 0x15, 0x00, Z_Zero);

        mem[0x0075] = 0x55;
        _t(&mem, 0x15, 0x55, 0);

        mem[0x0075] = 0x80;
        _t(&mem, 0x15, 0x80, N_Negative);

        mem[0x0075] = 0xAA;
        _t(&mem, 0x15, 0xAA, N_Negative);
    }

    //
    // LOAD - LDY
    //
    #[test]
    fn test_ldy_a0() { // LDY #$nn
        fn _t(mem: &[u8], exp_y: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            assert!(cpu.y == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.y == exp_y);
            assert!(cpu.p == exp_p);
        }

        let mem1 = &[LDY_A0, 0];
        let mem2 = &[LDY_A0, 0x55];
        let mem3 = &[LDY_A0, 0x80];
        let mem4 = &[LDY_A0, 0xAA];

        _t(mem1, 0, Z_Zero);
        _t(mem2, 0x55, 0);
        _t(mem3, 0x80, N_Negative);
        _t(mem4, 0xAA, N_Negative);
    }

    #[test]
    fn test_ldy_ac() { // LDY $nnnn
        fn _t(mem: &[u8], exp_y: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            assert!(cpu.y == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.y == exp_y);
            assert!(cpu.p == exp_p);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = LDY_AC;
        mem[1] = 0x20;
        mem[2] = 0x40;

        mem[0x4020] = 0;
        _t(&mem, 0x00, Z_Zero);

        mem[0x4020] = 0x80;
        _t(&mem, 0x80, N_Negative);

        mem[0x4020] = 0x55;
        _t(&mem, 0x55, 0);

        mem[0x4020] = 0xAA;
        _t(&mem, 0xAA, N_Negative);
    }

    #[test]
    fn test_ldy_bc() { // LDY $nnnn,X
        fn _t(mem: &[u8], x: u8, exp_y: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.x = x;
            assert!(cpu.y == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.y == exp_y);
            assert!(cpu.p == exp_p);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = LDY_BC;
        mem[1] = 0x60;
        mem[2] = 0x70;

        mem[0x7075] = 0;
        _t(&mem, 0x15, 0x00, Z_Zero);

        mem[0x7075] = 0x55;
        _t(&mem, 0x15, 0x55, 0);

        mem[0x7075] = 0x80;
        _t(&mem, 0x15, 0x80, N_Negative);

        mem[0x7075] = 0xAA;
        _t(&mem, 0x15, 0xAA, N_Negative);
    }

    #[test]
    fn test_ldy_a4() { // LDY $nn
        fn _t(mem: &[u8], exp_y: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            assert!(cpu.y == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.y == exp_y);
            assert!(cpu.p == exp_p);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = LDY_A4;
        mem[1] = 0x60;

        mem[0x0060] = 0;
        _t(&mem, 0x00, Z_Zero);

        mem[0x0060] = 0x55;
        _t(&mem, 0x55, 0);

        mem[0x0060] = 0x80;
        _t(&mem, 0x80, N_Negative);

        mem[0x0060] = 0xAA;
        _t(&mem, 0xAA, N_Negative);
    }

    #[test]
    fn test_ldy_b4() { // LDY $nn,X
        fn _t(mem: &[u8], x: u8, exp_y: u8, exp_p: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.x = x;
            assert!(cpu.y == 0);
            assert!(cpu.p == 0);
            cpu.step();
            assert!(cpu.y == exp_y);
            assert!(cpu.p == exp_p);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = LDY_B4;
        mem[1] = 0x60;

        mem[0x0075] = 0;
        _t(&mem, 0x15, 0x00, Z_Zero);

        mem[0x0075] = 0x55;
        _t(&mem, 0x15, 0x55, 0);

        mem[0x0075] = 0x80;
        _t(&mem, 0x15, 0x80, N_Negative);

        mem[0x0075] = 0xAA;
        _t(&mem, 0x15, 0xAA, N_Negative);
    }

    //
    // LOAD - STA
    //

    #[test]
    fn test_sta_8d() { // STA $nnnn
        fn _t(mem: &[u8], addr: usize, a: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.a = a;
            cpu.step();
            assert!(cpu.memory[addr] == a);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STA_8D;
        mem[1] = 0x60;
        mem[2] = 0x80;

        _t(&mem, 0x8060, 0x55);
        _t(&mem, 0x8060, 0x00);
        _t(&mem, 0x8060, 0x80);
        _t(&mem, 0x8060, 0xAA);
    }

    #[test]
    fn test_sta_9d() { // STA $nnnn,X
        fn _t(mem: &[u8], addr: usize, a: u8, x: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.a = a;
            cpu.x = x;
            cpu.step();
            assert!(cpu.memory[addr] == a);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STA_9D;
        mem[1] = 0x60;
        mem[2] = 0x80;

        _t(&mem, 0x8082, 0x55, 0x22);
        _t(&mem, 0x8082, 0x00, 0x22);
        _t(&mem, 0x8082, 0x80, 0x22);
        _t(&mem, 0x8082, 0xAA, 0x22);
    }

    #[test]
    fn test_sta_99() { // STA $nnnn,Y
        fn _t(mem: &[u8], addr: usize, a: u8, y: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.a = a;
            cpu.y = y;
            cpu.step();
            assert!(cpu.memory[addr] == a);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STA_99;
        mem[1] = 0x60;
        mem[2] = 0x80;

        _t(&mem, 0x8082, 0x55, 0x22);
        _t(&mem, 0x8082, 0x00, 0x22);
        _t(&mem, 0x8082, 0x80, 0x22);
        _t(&mem, 0x8082, 0xAA, 0x22);
    }

    #[test]
    fn test_sta_85() { // STA $nn
        fn _t(mem: &[u8], addr: usize, a: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.a = a;
            cpu.step();
            assert!(cpu.memory[addr] == a);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STA_85;
        mem[1] = 0x80;

        _t(&mem, 0x80, 0x55);
        _t(&mem, 0x80, 0x00);
        _t(&mem, 0x80, 0x80);
        _t(&mem, 0x80, 0xAA);
    }

    #[test]
    fn test_sta_95() { // STA $nn,X
        fn _t(mem: &[u8], addr: usize, a: u8, x: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.a = a;
            cpu.x = x;
            cpu.step();
            assert!(cpu.memory[addr] == a);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STA_95;
        mem[1] = 0x80;

        _t(&mem, 0xA2, 0x55, 0x22);
        _t(&mem, 0xA2, 0x00, 0x22);
        _t(&mem, 0xA2, 0x80, 0x22);
        _t(&mem, 0xA2, 0xAA, 0x22);
    }

    #[test]
    fn test_sta_81() { // STA ($nn,X)
        fn _t(mem: &[u8], addr: usize, a: u8, x: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.a = a;
            cpu.x = x;
            cpu.step();
            assert!(cpu.memory[addr] == a);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STA_81;
        mem[1] = 0x80;
        mem[0x98] = 0x22;
        mem[0x99] = 0x40;

        _t(&mem, 0x4022, 0x55, 0x18);
        _t(&mem, 0x4022, 0x00, 0x18);
        _t(&mem, 0x4022, 0x80, 0x18);
        _t(&mem, 0x4022, 0xAA, 0x18);
    }

    #[test]
    fn test_sta_91() { // STA ($nn),Y
        fn _t(mem: &[u8], addr: usize, a: u8, y: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.a = a;
            cpu.y = y;
            cpu.step();
            assert!(cpu.memory[addr] == a);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STA_91;
        mem[1] = 0x80;
        mem[0x80] = 0x22; // $4022 + $0018 = 403A
        mem[0x81] = 0x40;

        _t(&mem, 0x403A, 0x55, 0x18);
        _t(&mem, 0x403A, 0x00, 0x18);
        _t(&mem, 0x403A, 0x80, 0x18);
        _t(&mem, 0x403A, 0xAA, 0x18);
    }

    //
    // LOAD - STX
    //
    #[test]
    fn test_stx_8e() { // STX $nnnn
        fn _t(mem: &[u8], addr: usize, x: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.x = x;
            cpu.step();
            assert!(cpu.memory[addr] == x);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STX_8E;
        mem[1] = 0x60;
        mem[2] = 0x80;

        _t(&mem, 0x8060, 0x55);
        _t(&mem, 0x8060, 0x00);
        _t(&mem, 0x8060, 0x80);
        _t(&mem, 0x8060, 0xAA);
    }

    #[test]
    fn test_stx_86() { // STX $nn
        fn _t(mem: &[u8], addr: usize, x: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.x = x;
            cpu.step();
            assert!(cpu.memory[addr] == x);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STX_86;
        mem[1] = 0xC0;

        _t(&mem, 0x00C0, 0x55);
        _t(&mem, 0x00C0, 0x00);
        _t(&mem, 0x00C0, 0x80);
        _t(&mem, 0x00C0, 0xAA);
    }

    #[test]
    fn test_stx_96() { // STX $nn,Y
        fn _t(mem: &[u8], addr: usize, x: u8, y: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.x = x;
            cpu.y = y;
            cpu.step();
            assert!(cpu.memory[addr] == x);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STX_96;
        mem[1] = 0xC0;

        _t(&mem, 0x00E2, 0x55, 0x22);
        _t(&mem, 0x00D0, 0x00, 0x10);
        _t(&mem, 0x00D5, 0x80, 0x15);
        _t(&mem, 0x0020, 0xAA, 0x60); // no crossing of zero-page boundaries
    }

    //
    // LOAD - STY
    //

    #[test]
    fn test_sty_8c() { // STY $nnnn
        fn _t(mem: &[u8], addr: usize, y: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.y = y;
            cpu.step();
            assert!(cpu.memory[addr] == y);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STY_8C;
        mem[1] = 0x60;
        mem[2] = 0x80;

        _t(&mem, 0x8060, 0x55);
        _t(&mem, 0x8060, 0x00);
        _t(&mem, 0x8060, 0x80);
        _t(&mem, 0x8060, 0xAA);
    }

    #[test]
    fn test_sty_84() { // STY $nn
        fn _t(mem: &[u8], addr: usize, y: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.y = y;
            cpu.step();
            assert!(cpu.memory[addr] == y);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STY_84;
        mem[1] = 0xC0;

        _t(&mem, 0x00C0, 0x55);
        _t(&mem, 0x00C0, 0x00);
        _t(&mem, 0x00C0, 0x80);
        _t(&mem, 0x00C0, 0xAA);
    }

    #[test]
    fn test_sty_94() { // STY $nn,X
        fn _t(mem: &[u8], addr: usize, y: u8, x: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, mem);
            cpu.x = x;
            cpu.y = y;
            cpu.step();
            assert!(cpu.memory[addr] == y);
        }

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = STY_94;
        mem[1] = 0xC0;

        _t(&mem, 0x00E2, 0x55, 0x22);
        _t(&mem, 0x00D0, 0x00, 0x10);
        _t(&mem, 0x00D5, 0x80, 0x15);
        _t(&mem, 0x0020, 0xAA, 0x60); // no crossing of zero-page boundaries
    }

    //
    // NOP
    //
    #[test]
    fn test_nop() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[NOP_EA]);
        cpu.step();

        let fresh = Cpu::new();
        assert_eq!(cpu.a, fresh.a);
        assert_eq!(cpu.x, fresh.x);
        assert_eq!(cpu.y, fresh.y);
        assert_eq!(cpu.p, fresh.p);
        assert_eq!(cpu.s, fresh.s);
        assert_eq!(cpu.pc, fresh.pc + 1);
    }

    //
    // INC
    //
    #[test]
    fn test_inc_ee() { // INC $nnnn
        let mut cpu = Cpu::new();
        let mut expected: u8 = 200u8;

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = INC_EE; // INC $4000
        mem[1] = 0x00;
        mem[2] = 0x40;
        mem[0x4000] = 200;
        cpu.patch_memory(0, &mem);

        for _ in 1..512 {
            cpu.pc = 0x0000;
            cpu.step();
            expected = expected.wrapping_add(1);

            assert_eq!(cpu.memory[0x4000], expected);

            let expected_z: bool = expected == 0;
            let expected_n = (expected & 0x80) > 0;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative > 0, expected_n);
        }
    }

    #[test]
    fn test_inc_fe() { // INC $nnnn,X
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[INC_FE, 0x20, 0x40]); // INC $4020
        cpu.memory[0x4055] = 200;
        cpu.x = 0x35;
        let mut expected: u8 = 200u8;

        for _ in 1..512 {
            cpu.pc = 0x0000;
            cpu.step();
            expected = expected.wrapping_add(1);

            assert_eq!(cpu.memory[0x4055], expected);

            let expected_z: bool = expected == 0;
            let expected_n = (expected & 0x80) > 0;
            assert_eq!(cpu.is_zero(), expected_z);
            assert_eq!(cpu.is_negative(), expected_n);
        }
    }

    #[test]
    fn test_inc_e6() { // INC $nn
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[INC_E6, 0x20]); // INC $20
        cpu.memory[0x0020] = 200;
        let mut expected: u8 = 200u8;

        for _ in 1..512 {
            cpu.pc = 0x0000;
            cpu.step();
            expected = expected.wrapping_add(1);

            assert_eq!(cpu.memory[0x0020], expected);

            let expected_z: bool = expected == 0;
            let expected_n = (expected & 0x80) > 0;
            assert_eq!(cpu.is_zero(), expected_z);
            assert_eq!(cpu.is_negative(), expected_n);
        }
    }

    #[test]
    fn test_inc_f6() { // INC $nn,X
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[INC_F6, 0x20]); // INC $20
        cpu.memory[0x0055] = 200;
        cpu.x = 0x35;
        let mut expected: u8 = 200u8;

        for _ in 1..512 {
            cpu.pc = 0x0000;
            cpu.step();
            expected = expected.wrapping_add(1);

            assert_eq!(cpu.memory[0x0055], expected);

            let expected_z: bool = expected == 0;
            let expected_n = (expected & 0x80) > 0;
            assert_eq!(cpu.is_zero(), expected_z);
            assert_eq!(cpu.is_negative(), expected_n);
        }
    }

    //
    // INX
    //
    #[test]
    fn test_inx() {
        let mut cpu = Cpu::new();
        let mem: [u8; 512] = [INX_E8; 512];
        cpu.patch_memory(0, &mem);
        let mut expected_x: u16 = 0;

        assert_eq!(cpu.x, expected_x as u8);

        cpu.step(); // INX
        expected_x += 1;

        assert_eq!(cpu.x, expected_x as u8);
        assert_eq!(cpu.p, 0);

        for _ in 1..512 {
            cpu.step(); // INX
            expected_x += 1;

            assert_eq!(cpu.x, expected_x as u8);

            let expected_z: bool = if (expected_x as u8) == 0 { true } else { false };
            let expected_n = expected_x & 0x80 > 0;
            assert_eq!(cpu.is_zero(), expected_z);
            assert_eq!(cpu.is_negative(), expected_n);
        }
    }

    #[test]
    fn test_iny() {
        let mut cpu = Cpu::new();
        let mem: [u8; 512] = [INY_C8; 512];
        cpu.patch_memory(0, &mem);
        let mut expected_y: u16 = 0;

        assert_eq!(cpu.y, expected_y as u8);

        cpu.step(); // INY
        expected_y += 1;

        assert_eq!(cpu.y, expected_y as u8);
        assert_eq!(cpu.p, 0);

        for _ in 1..512 {
            cpu.step(); // INY
            expected_y += 1;

            assert_eq!(cpu.y, expected_y as u8);

            let expected_z: bool = if (expected_y as u8) == 0 { true } else { false };
            let expected_n = expected_y & 0x80 > 0;
            assert_eq!(cpu.is_zero(), expected_z);
            assert_eq!(cpu.is_negative(), expected_n);
        }
    }

    //
    // ADC
    //
    // FIXME: Add decimal mode testing
    #[test]
    fn test_adc_69() { // ADC #$nn
        fn _t(a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = ADC_69;
            mem[1] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   a  mem[1]  carry  exp_a  flags
        _t(120,     12,  true,   133, N_Negative);
        //_t(-80,     40, false,   -40, N_Negative);
        _t(  0,      0, false,     0, Z_Zero);
        _t(  0,      0,  true,     1, 0);
        _t(  0,      3, false,     3, 0);
        _t(  4,      0, false,     4, 0);
        _t(  4,      0,  true,     5, 0);
        _t(120,    180, false,    44, C_Carry);
        _t(120,    180,  true,    45, C_Carry);
        _t(100,    100, false,   200, N_Negative);
        _t(255,    255,  true,   255, N_Negative|C_Carry);
        _t(128,    128, false,     0, Z_Zero|C_Carry);
        _t(128,    128,  true,     1, C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_adc_6d() { // ADC $nnnn
        fn _t(addr: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = ADC_6D;
            mem[1] = addr as u8;
            mem[2] = (addr >> 8) as u8;
            mem[addr] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr    a  mem[1]  carry  exp_a  flags
        _t(0x1024, 120,     12,  true,   133, N_Negative);
        //_t(-80,     40, false,   -40, N_Negative);
        _t(0x8060,   0,      0, false,     0, Z_Zero);
        _t(0x0512,   0,      0,  true,     1, 0);
        _t(0x8000,   0,      3, false,     3, 0);
        _t(0xAAAA,   4,      0, false,     4, 0);
        _t(0x2828,   4,      0,  true,     5, 0);
        _t(0x7373, 120,    180, false,    44, C_Carry);
        _t(0x1234, 120,    180,  true,    45, C_Carry);
        _t(0x0060, 100,    100, false,   200, N_Negative);
        _t(0x00ff, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 128,    128, false,     0, Z_Zero|C_Carry);
        _t(0x9030, 128,    128,  true,     1, C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_adc_7d() { // ADC $nnnn,X
        fn _t(addr: usize, x: u8, addr2: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = ADC_7D;
            mem[1] = addr as u8;
            mem[2] = (addr >> 8) as u8;
            mem[addr2] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr     x   addr2,   a  mem[1]  carry  exp_a  flags
        _t(0x1024, 0x20, 0x1044, 120,     12,  true,   133, N_Negative);
        //_t(-80,  0x00, 0x0000,    40, false,   -40, N_Negative);
        _t(0x8060, 0x44, 0x80a4,   0,      0, false,     0, Z_Zero);
        _t(0x0512, 0x00, 0x0512,   0,      0,  true,     1, 0);
        _t(0x8000, 0x13, 0x8013,   0,      3, false,     3, 0);
        _t(0xAAAA, 0x1f, 0xaac9,   4,      0, false,     4, 0);
        _t(0x2828, 0x0b, 0x2833,   4,      0,  true,     5, 0);
        _t(0x7373, 0x15, 0x7388, 120,    180, false,    44, C_Carry);
        _t(0x1234, 0x20, 0x1254, 120,    180,  true,    45, C_Carry);
        _t(0x0060, 0x33, 0x0093, 100,    100, false,   200, N_Negative);
        _t(0x00ff, 0x14, 0x0113, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 0x44, 0x00c4, 128,    128, false,     0, Z_Zero|C_Carry);
        _t(0x9030, 0x11, 0x9041, 128,    128,  true,     1, C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_adc_79() { // ADC $nnnn,Y
        fn _t(addr: usize, y: u8, addr2: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = ADC_79;
            mem[1] = addr as u8;
            mem[2] = (addr >> 8) as u8;
            mem[addr2] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr     y   addr2,   a  mem[1]  carry  exp_a  flags
        _t(0x1024, 0x20, 0x1044, 120,     12,  true,   133, N_Negative);
        //_t(-80,  0x00, 0x0000,    40, false,   -40, N_Negative);
        _t(0x8060, 0x44, 0x80a4,   0,      0, false,     0, Z_Zero);
        _t(0x0512, 0x00, 0x0512,   0,      0,  true,     1, 0);
        _t(0x8000, 0x13, 0x8013,   0,      3, false,     3, 0);
        _t(0xAAAA, 0x1f, 0xaac9,   4,      0, false,     4, 0);
        _t(0x2828, 0x0b, 0x2833,   4,      0,  true,     5, 0);
        _t(0x7373, 0x15, 0x7388, 120,    180, false,    44, C_Carry);
        _t(0x1234, 0x20, 0x1254, 120,    180,  true,    45, C_Carry);
        _t(0x0060, 0x33, 0x0093, 100,    100, false,   200, N_Negative);
        _t(0x00ff, 0x14, 0x0113, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 0x44, 0x00c4, 128,    128, false,     0, Z_Zero|C_Carry);
        _t(0x9030, 0x11, 0x9041, 128,    128,  true,     1, C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_adc_65() { // ADC $nn
        fn _t(addr: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = ADC_65;
            mem[1] = addr as u8;
            mem[addr] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr    a  mem[1]  carry  exp_a  flags
        _t(0x0024, 120,     12,  true,   133, N_Negative);
        //_t(-80,     40, false,   -40, N_Negative);
        _t(0x0060,   0,      0, false,     0, Z_Zero);
        _t(0x0012,   0,      0,  true,     1, 0);
        _t(0x0020,   0,      3, false,     3, 0);
        _t(0x00AA,   4,      0, false,     4, 0);
        _t(0x0028,   4,      0,  true,     5, 0);
        _t(0x0073, 120,    180, false,    44, C_Carry);
        _t(0x0034, 120,    180,  true,    45, C_Carry);
        _t(0x0060, 100,    100, false,   200, N_Negative);
        _t(0x00ff, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 128,    128, false,     0, Z_Zero|C_Carry);
        _t(0x0030, 128,    128,  true,     1, C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_adc_75() { // ADC $nn,X
        fn _t(addr: usize, x: u8, addr2: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = ADC_75;
            mem[1] = addr as u8;
            mem[addr2] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr     x   addr2    a  mem[1]  carry  exp_a  flags
        _t(0x0024, 0x00, 0x0024, 120,     12,  true,   133, N_Negative);
        _t(0x0060, 0x13, 0x0073,   0,      0, false,     0, Z_Zero);
        _t(0x0012, 0x22, 0x0034,   0,      0,  true,     1, 0);
        _t(0x0020, 0x05, 0x0025,   0,      3, false,     3, 0);
        _t(0x00AA, 0xFF, 0x0099,   4,      0, false,     4, 0);
        _t(0x0028, 0x10, 0x0038,   4,      0,  true,     5, 0);
        _t(0x0073, 0x14, 0x0087, 120,    180, false,    44, C_Carry);
        _t(0x0034, 0x01, 0x0035, 120,    180,  true,    45, C_Carry);
        _t(0x0060, 0x60, 0x00C0, 100,    100, false,   200, N_Negative);
        _t(0x00ff, 0x22, 0x0021, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 0x88, 0x0008, 128,    128, false,     0, Z_Zero|C_Carry);
        _t(0x0030, 0x05, 0x0035, 128,    128,  true,     1, C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_adc_61() { // ADC ($nn,X)
        fn _t(addr: usize, x: u8, addr2: usize, addr3: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = ADC_61;
            mem[1] = addr as u8;
            assert!(addr2 & !0xff == 0); // addr2 should be in zero page
            // FIXME: what if addr2 is 0xff ??? How 6502 should work ???
            mem[addr2] = addr3 as u8;
            mem[(addr2+1) & 0xff] = (addr3 >> 8) as u8;
            mem[addr3] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr     x   addr2   addr3    a  mem[1]  carry  exp_a  flags
        _t(0x0024, 0x00, 0x0024, 0x8080, 120,     12,  true,   133, N_Negative);
        _t(0x0060, 0x13, 0x0073, 0x0505,   0,      0, false,     0, Z_Zero);
        _t(0x0012, 0x22, 0x0034, 0xaaaa,   0,      0,  true,     1, 0);
        _t(0x0020, 0x05, 0x0025, 0xbbbb,   0,      3, false,     3, 0);
        _t(0x00AA, 0xff, 0x00a9, 0x22aa,   4,      0, false,     4, 0);
        _t(0x0028, 0x10, 0x0038, 0xdddd,   4,      0,  true,     5, 0);
        _t(0x0073, 0x14, 0x0087, 0xabcd, 120,    180, false,    44, C_Carry);
        _t(0x0034, 0x01, 0x0035, 0xbcde, 120,    180,  true,    45, C_Carry);
        _t(0x0060, 0x60, 0x00c0, 0xcdef, 100,    100, false,   200, N_Negative);
        _t(0x00ff, 0x22, 0x0021, 0xdefa, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 0x88, 0x0008, 0xefab, 128,    128, false,     0, Z_Zero|C_Carry);
        _t(0x0030, 0x05, 0x0035, 0xfabc, 128,    128,  true,     1, C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_adc_71() { // ADC ($nn),Y
        fn _t(addr: usize, addr2: usize, y: u8, addr3: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = ADC_71;
            mem[1] = addr as u8;
            mem[addr] = addr2 as u8;
            mem[(addr + 1) & 0xff] = (addr2 >> 8) as u8;
            mem[addr3] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();
            cpu._dump_memory();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr   addr2     y   addr3    a  mem[1]  carry  exp_a  flags
        _t(0x0024, 0x4010, 0x00, 0x4010, 120,     12,  true,   133, N_Negative);
        _t(0x0060, 0x4000, 0x13, 0x4013,   0,      0, false,     0, Z_Zero);
        _t(0x0012, 0x4000, 0x22, 0x4022,   0,      0,  true,     1, 0);
        _t(0x0020, 0x4000, 0x05, 0x4005,   0,      3, false,     3, 0);
        _t(0x00AA, 0x4000, 0xff, 0x40ff,   4,      0, false,     4, 0);
        _t(0x0028, 0x4000, 0x10, 0x4010,   4,      0,  true,     5, 0);
        _t(0x0073, 0x4000, 0x14, 0x4014, 120,    180, false,    44, C_Carry);
        _t(0x0034, 0x40ff, 0x01, 0x4100, 120,    180,  true,    45, C_Carry);
        _t(0x0060, 0x40ff, 0x60, 0x415f, 100,    100, false,   200, N_Negative);
        _t(0x00f0, 0x40ff, 0x22, 0x4121, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 0x40ff, 0x88, 0x4187, 128,    128, false,     0, Z_Zero|C_Carry);
        _t(0x0030, 0x40ff, 0x05, 0x4104, 128,    128,  true,     1, C_Carry);

        // FIXME: test for overflows
    }

    //
    // SBC
    //
    // FIXME: Add decimal mode testing
    #[test]
    fn test_sbc_e9() { // SBC #$nn
        fn _t(a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = SBC_E9;
            mem[1] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert_eq!(cpu.a, exp_a);
            assert_eq!(cpu.p, exp_flags);
        }

        //   a       v  carry  exp_a  flags
        _t(120,     12,  true,   107, 0);
        _t(  0,      0, false,     0, Z_Zero);
        _t(  0,      0,  true,   255, N_Negative|C_Carry);
        _t(  3,      0, false,     3, 0);
        _t(  4,      0,  true,     3, 0);
        _t(  4,      3,  true,     0, Z_Zero);
        _t(180,    120, false,    60, 0);
        _t(180,    120,  true,    59, 0);
        _t(100,    101, false,   255, N_Negative|C_Carry);
        _t(255,    255,  true,   255, N_Negative|C_Carry);
        _t(128,    128, false,     0, Z_Zero);
        _t(128,    128,  true,   255, N_Negative|C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_sbc_ed() { // SBC $nnnn
        fn _t(addr: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = SBC_ED;
            mem[1] = addr as u8;
            mem[2] = (addr >> 8) as u8;
            mem[addr] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert_eq!(cpu.a, exp_a);
            assert_eq!(cpu.p, exp_flags);
        }

        //   addr    a       v  carry  exp_a  flags
        _t(0x1024, 120,     12,  true,   107, 0);
        _t(0x8060,   0,      0, false,     0, Z_Zero);
        _t(0x0512,   0,      0,  true,   255, N_Negative|C_Carry);
        _t(0x8000,   3,      0, false,     3, 0);
        _t(0xAAAA,   4,      0,  true,     3, 0);
        _t(0x2828,   4,      3,  true,     0, Z_Zero);
        _t(0x7373, 180,    120, false,    60, 0);
        _t(0x1234, 180,    120,  true,    59, 0);
        _t(0x0060, 100,    101, false,   255, N_Negative|C_Carry);
        _t(0x00ff, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 128,    128, false,     0, Z_Zero);
        _t(0x9030, 128,    128,  true,   255, N_Negative|C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_sbc_fd() { // SBC $nnnn,X
        fn _t(addr: usize, x: u8, addr2: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = SBC_FD;
            mem[1] = addr as u8;
            mem[2] = (addr >> 8) as u8;
            mem[addr2] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr     x   addr2,   a       v  carry  exp_a  flags
        _t(0x1024, 0x20, 0x1044, 120,     12,  true,   107, 0);
        _t(0x8060, 0x44, 0x80a4,   0,      0, false,     0, Z_Zero);
        _t(0x0512, 0x00, 0x0512,   0,      0,  true,   255, N_Negative|C_Carry);
        _t(0x8000, 0x13, 0x8013,   3,      0, false,     3, 0);
        _t(0xAAAA, 0x1f, 0xaac9,   4,      0,  true,     3, 0);
        _t(0x2828, 0x0b, 0x2833,   4,      3,  true,     0, Z_Zero);
        _t(0x7373, 0x15, 0x7388, 180,    120, false,    60, 0);
        _t(0x1234, 0x20, 0x1254, 180,    120,  true,    59, 0);
        _t(0x0060, 0x33, 0x0093, 100,    101, false,   255, N_Negative|C_Carry);
        _t(0x00ff, 0x14, 0x0113, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 0x44, 0x00c4, 128,    128, false,     0, Z_Zero);
        _t(0x9030, 0x11, 0x9041, 128,    128,  true,   255, N_Negative|C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_sbc_f9() { // SBC $nnnn,Y
        fn _t(addr: usize, y: u8, addr2: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = SBC_F9;
            mem[1] = addr as u8;
            mem[2] = (addr >> 8) as u8;
            mem[addr2] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr     y   addr2,   a       v  carry  exp_a  flags
        _t(0x1024, 0x20, 0x1044, 120,     12,  true,   107, 0);
        _t(0x8060, 0x44, 0x80a4,   0,      0, false,     0, Z_Zero);
        _t(0x0512, 0x00, 0x0512,   0,      0,  true,   255, N_Negative|C_Carry);
        _t(0x8000, 0x13, 0x8013,   3,      0, false,     3, 0);
        _t(0xAAAA, 0x1f, 0xaac9,   4,      0,  true,     3, 0);
        _t(0x2828, 0x0b, 0x2833,   4,      3,  true,     0, Z_Zero);
        _t(0x7373, 0x15, 0x7388, 180,    120, false,    60, 0);
        _t(0x1234, 0x20, 0x1254, 180,    120,  true,    59, 0);
        _t(0x0060, 0x33, 0x0093, 100,    101, false,   255, N_Negative|C_Carry);
        _t(0x00ff, 0x14, 0x0113, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 0x44, 0x00c4, 128,    128, false,     0, Z_Zero);
        _t(0x9030, 0x11, 0x9041, 128,    128,  true,   255, N_Negative|C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_sbc_e5() { // SBC $nn
        fn _t(addr: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = SBC_E5;
            mem[1] = addr as u8;
            mem[addr] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr    a       v  carry  exp_a  flags
        _t(0x0024, 120,     12,  true,   107, 0);
        _t(0x0060,   0,      0, false,     0, Z_Zero);
        _t(0x0012,   0,      0,  true,   255, N_Negative|C_Carry);
        _t(0x0020,   3,      0, false,     3, 0);
        _t(0x00AA,   4,      0,  true,     3, 0);
        _t(0x0028,   4,      3,  true,     0, Z_Zero);
        _t(0x0073, 180,    120, false,    60, 0);
        _t(0x0034, 180,    120,  true,    59, 0);
        _t(0x0060, 100,    101, false,   255, N_Negative|C_Carry);
        _t(0x00ff, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 128,    128, false,     0, Z_Zero);
        _t(0x0030, 128,    128,  true,   255, N_Negative|C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_sbc_f5() { // SBC $nn,X
        fn _t(addr: usize, x: u8, addr2: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = SBC_F5;
            mem[1] = addr as u8;
            mem[addr2] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr     x   addr2    a       v  carry  exp_a  flags
        _t(0x0024, 0x00, 0x0024, 120,     12,  true,   107, 0);
        _t(0x0060, 0x13, 0x0073,   0,      0, false,     0, Z_Zero);
        _t(0x0012, 0x22, 0x0034,   0,      0,  true,   255, N_Negative|C_Carry);
        _t(0x0020, 0x05, 0x0025,   3,      0, false,     3, 0);
        _t(0x00AA, 0xFF, 0x0099,   4,      0,  true,     3, 0);
        _t(0x0028, 0x10, 0x0038,   4,      3,  true,     0, Z_Zero);
        _t(0x0073, 0x14, 0x0087, 180,    120, false,    60, 0);
        _t(0x0034, 0x01, 0x0035, 180,    120,  true,    59, 0);
        _t(0x0060, 0x60, 0x00C0, 100,    101, false,   255, N_Negative|C_Carry);
        _t(0x00ff, 0x22, 0x0021, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 0x88, 0x0008, 128,    128, false,     0, Z_Zero);
        _t(0x0030, 0x05, 0x0035, 128,    128,  true,   255, N_Negative|C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_sbc_e1() { // SBC ($nn,X)
        fn _t(addr: usize, x: u8, addr2: usize, addr3: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = SBC_E1;
            mem[1] = addr as u8;
            assert!(addr2 & !0xff == 0); // addr2 should be in zero page
            // FIXME: what if addr2 is 0xff ??? How 6502 should work ???
            mem[addr2] = addr3 as u8;
            mem[(addr2+1) & 0xff] = (addr3 >> 8) as u8;
            mem[addr3] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();

            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr     x   addr2   addr3    a       v  carry  exp_a  flags
        _t(0x0024, 0x00, 0x0024, 0x8080, 120,     12,  true,   107, 0);
        _t(0x0060, 0x13, 0x0073, 0x0505,   0,      0, false,     0, Z_Zero);
        _t(0x0012, 0x22, 0x0034, 0xaaaa,   0,      0,  true,   255, N_Negative|C_Carry);
        _t(0x0020, 0x05, 0x0025, 0xbbbb,   3,      0, false,     3, 0);
        _t(0x00AA, 0xff, 0x00a9, 0x22aa,   4,      0,  true,     3, 0);
        _t(0x0028, 0x10, 0x0038, 0xdddd,   4,      3,  true,     0, Z_Zero);
        _t(0x0073, 0x14, 0x0087, 0xabcd, 180,    120, false,    60, 0);
        _t(0x0034, 0x01, 0x0035, 0xbcde, 180,    120,  true,    59, 0);
        _t(0x0060, 0x60, 0x00c0, 0xcdef, 100,    101, false,   255, N_Negative|C_Carry);
        _t(0x00ff, 0x22, 0x0021, 0xdefa, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 0x88, 0x0008, 0xefab, 128,    128, false,     0, Z_Zero);
        _t(0x0030, 0x05, 0x0035, 0xfabc, 128,    128,  true,   255, N_Negative|C_Carry);

        // FIXME: test for overflows
    }

    #[test]
    fn test_sbc_f1() { // SBC ($nn),Y
        fn _t(addr: usize, addr2: usize, y: u8, addr3: usize, a: u8, v: u8, carry: bool, exp_a: u8, exp_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            cpu.update_carry(carry);

            let mut mem = [0u8; MEM_SZ];
            mem[0] = SBC_F1;
            mem[1] = addr as u8;
            mem[addr] = addr2 as u8;
            mem[(addr + 1) & 0xff] = (addr2 >> 8) as u8;
            mem[addr3] = v;
            cpu.patch_memory(0, &mem);

            cpu.step();
            cpu._dump_memory();

            println!("cpu.a: {}", cpu.a);
            println!("exp_a: {}", exp_a);
            assert!(cpu.a == exp_a);
            assert!(cpu.p == exp_flags);
        }

        //   addr   addr2     y   addr3    a       v  carry  exp_a  flags
        _t(0x0024, 0x4010, 0x00, 0x4010, 120,     12,  true,   107, 0);
        _t(0x0060, 0x4000, 0x13, 0x4013,   0,      0, false,     0, Z_Zero);
        _t(0x0012, 0x4000, 0x22, 0x4022,   0,      0,  true,   255, N_Negative|C_Carry);
        _t(0x0020, 0x4000, 0x05, 0x4005,   3,      0, false,     3, 0);
        _t(0x00AA, 0x4000, 0xff, 0x40ff,   4,      0,  true,     3, 0);
        _t(0x0028, 0x4000, 0x10, 0x4010,   4,      3,  true,     0, Z_Zero);
        _t(0x0073, 0x4000, 0x14, 0x4014, 180,    120, false,    60, 0);
        _t(0x0034, 0x40ff, 0x01, 0x4100, 180,    120,  true,    59, 0);
        _t(0x0060, 0x40ff, 0x60, 0x415f, 100,    101, false,   255, N_Negative|C_Carry);
        _t(0x00f0, 0x40ff, 0x22, 0x4121, 255,    255,  true,   255, N_Negative|C_Carry);
        _t(0x0080, 0x40ff, 0x88, 0x4187, 128,    128, false,     0, Z_Zero);
        _t(0x0030, 0x40ff, 0x05, 0x4104, 128,    128,  true,   255, N_Negative|C_Carry);

        // FIXME: test for overflows
    }

    //
    // CMP
    //
    #[test]
    fn test_cmp_c9() { // CMP #$nn
        fn _t(a: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CMP_C9;
            mem[1] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    a   val  flags
        _t(0x00, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x40, C_Carry);
        _t(0x40, 0x50, N_Negative);
        _t(0x00, 0x50, N_Negative);
        _t(0x20, 0x00, C_Carry);
        _t(0x55, 0x55, Z_Zero | C_Carry);
    }

    #[test]
    fn test_cmp_cd() { // CMP $nnnn
        fn _t(a: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CMP_CD;
            mem[1] = 0x30;
            mem[2] = 0x10;
            mem[0x1030] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    a   val  flags
        _t(0x00, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x40, C_Carry);
        _t(0x40, 0x50, N_Negative);
        _t(0x00, 0x50, N_Negative);
        _t(0x20, 0x00, C_Carry);
        _t(0x55, 0x55, Z_Zero | C_Carry);
    }
    #[test]
    fn test_cmp_dd() { // CMP $nnnn,X
        fn _t(a: u8, x: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CMP_DD;
            mem[1] = 0x10;
            mem[2] = 0x10;
            mem[0x1030] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    a     x   val  flags
        _t(0x00, 0x20, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x20, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x20, 0x40, C_Carry);
        _t(0x40, 0x20, 0x50, N_Negative);
        _t(0x00, 0x20, 0x50, N_Negative);
        _t(0x20, 0x20, 0x00, C_Carry);
        _t(0x55, 0x20, 0x55, Z_Zero | C_Carry);
    }
    #[test]
    fn test_cmp_d9() { // CMP $nnnn,Y
        fn _t(a: u8, y: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CMP_D9;
            mem[1] = 0x10;
            mem[2] = 0x10;
            mem[0x1030] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    a     y   val  flags
        _t(0x00, 0x20, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x20, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x20, 0x40, C_Carry);
        _t(0x40, 0x20, 0x50, N_Negative);
        _t(0x00, 0x20, 0x50, N_Negative);
        _t(0x20, 0x20, 0x00, C_Carry);
        _t(0x55, 0x20, 0x55, Z_Zero | C_Carry);
    }

    #[test]
    fn test_cmp_c5() { // CMP $nn
        fn _t(a: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CMP_C5;
            mem[1] = 0x30;
            mem[0x0030] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    a   val  flags
        _t(0x00, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x40, C_Carry);
        _t(0x40, 0x50, N_Negative);
        _t(0x00, 0x50, N_Negative);
        _t(0x20, 0x00, C_Carry);
        _t(0x55, 0x55, Z_Zero | C_Carry);
    }

    #[test]
    fn test_cmp_d5() { // CMP $nn,X
        fn _t(a: u8, x: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CMP_D5;
            mem[1] = 0x30;
            mem[0x0080] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    a     x   val  flags
        _t(0x00, 0x50, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x50, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x50, 0x40, C_Carry);
        _t(0x40, 0x50, 0x50, N_Negative);
        _t(0x00, 0x50, 0x50, N_Negative);
        _t(0x20, 0x50, 0x00, C_Carry);
        _t(0x55, 0x50, 0x55, Z_Zero | C_Carry);
    }

    #[test]
    fn test_cmp_c1() { // CMP ($nn,X)
        fn _t(a: u8, x: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CMP_C1;
            mem[1] = 0x30;
            mem[0x0080] = 0x50;
            mem[0x0081] = 0x46;
            mem[0x4650] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    a     x   val  flags
        _t(0x00, 0x50, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x50, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x50, 0x40, C_Carry);
        _t(0x40, 0x50, 0x50, N_Negative);
        _t(0x00, 0x50, 0x50, N_Negative);
        _t(0x20, 0x50, 0x00, C_Carry);
        _t(0x55, 0x50, 0x55, Z_Zero | C_Carry);
    }

    #[test]
    fn test_cmp_d1() { // CMP ($nn),Y
        fn _t(a: u8, y: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CMP_D1;
            mem[1] = 0x30;
            mem[0x0030] = 0x00;
            mem[0x0031] = 0x40;
            mem[0x4028] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    a     y   val  flags
        _t(0x00, 0x28, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x28, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x28, 0x40, C_Carry);
        _t(0x40, 0x28, 0x50, N_Negative);
        _t(0x00, 0x28, 0x50, N_Negative);
        _t(0x20, 0x28, 0x00, C_Carry);
        _t(0x55, 0x28, 0x55, Z_Zero | C_Carry);
    }

    //
    // CPX
    //
    #[test]
    fn test_cpx_e0() { // CPX #$nn
        fn _t(x: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.x = x;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CPX_E0;
            mem[1] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    x   val  flags
        _t(0x00, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x40, C_Carry);
        _t(0x40, 0x50, N_Negative);
        _t(0x00, 0x50, N_Negative);
        _t(0x20, 0x00, C_Carry);
        _t(0x55, 0x55, Z_Zero | C_Carry);
    }

    #[test]
    fn test_cpx_ec() { // CPX $nnnn
        fn _t(x: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.x = x;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CPX_EC;
            mem[1] = 0x30;
            mem[2] = 0x10;
            mem[0x1030] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    x   val  flags
        _t(0x00, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x40, C_Carry);
        _t(0x40, 0x50, N_Negative);
        _t(0x00, 0x50, N_Negative);
        _t(0x20, 0x00, C_Carry);
        _t(0x55, 0x55, Z_Zero | C_Carry);
    }

    #[test]
    fn test_cpx_e4() { // CPX $nn
        fn _t(x: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.x = x;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CPX_E4;
            mem[1] = 0x30;
            mem[0x0030] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    x   val  flags
        _t(0x00, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x40, C_Carry);
        _t(0x40, 0x50, N_Negative);
        _t(0x00, 0x50, N_Negative);
        _t(0x20, 0x00, C_Carry);
        _t(0x55, 0x55, Z_Zero | C_Carry);
    }

    //
    // CPY
    //
    #[test]
    fn test_cpy_c0() { // CPY #$nn
        fn _t(y: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.y = y;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CPY_C0;
            mem[1] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    y   val  flags
        _t(0x00, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x40, C_Carry);
        _t(0x40, 0x50, N_Negative);
        _t(0x00, 0x50, N_Negative);
        _t(0x20, 0x00, C_Carry);
        _t(0x55, 0x55, Z_Zero | C_Carry);
    }

    #[test]
    fn test_cpy_cc() { // CPY $nnnn
        fn _t(y: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.y = y;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CPY_CC;
            mem[1] = 0x30;
            mem[2] = 0x10;
            mem[0x1030] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    y   val  flags
        _t(0x00, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x40, C_Carry);
        _t(0x40, 0x50, N_Negative);
        _t(0x00, 0x50, N_Negative);
        _t(0x20, 0x00, C_Carry);
        _t(0x55, 0x55, Z_Zero | C_Carry);
    }

    #[test]
    fn test_cpy_c4() { // CPY $nn
        fn _t(y: u8, val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.y = y;
            let mut mem = [0u8; MEM_SZ];
            mem[0] = CPY_C4;
            mem[1] = 0x30;
            mem[0x0030] = val;

            cpu.patch_memory(0, &mem);
            cpu.step();

            assert!(cpu.p == expected_flags);
        }

        //    y   val  flags
        _t(0x00, 0x00, Z_Zero | C_Carry);
        _t(0x80, 0x00, N_Negative | C_Carry);
        _t(0x50, 0x40, C_Carry);
        _t(0x40, 0x50, N_Negative);
        _t(0x00, 0x50, N_Negative);
        _t(0x20, 0x00, C_Carry);
        _t(0x55, 0x55, Z_Zero | C_Carry);
    }

    //
    // DEC
    //
    #[test]
    fn test_dec_ce() { // DEC $nnnn
        let mut cpu = Cpu::new();
        let mut expected: u8 = 200u8;

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = DEC_CE; // DEC $4000
        mem[1] = 0x00;
        mem[2] = 0x40;
        mem[0x4000] = 200;
        cpu.patch_memory(0, &mem);

        for _ in 1..512 {
            cpu.pc = 0x0000;
            cpu.step();
            expected = expected.wrapping_sub(1);

            assert_eq!(cpu.memory[0x4000], expected);

            let expected_z: bool = expected == 0;
            let expected_n = (expected & 0x80) > 0;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative > 0, expected_n);
        }
    }

    #[test]
    fn test_dec_de() { // DEC $nnnn,X
        let mut cpu = Cpu::new();
        cpu.x = 0x35;
        let mut expected: u8 = 200u8;

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = DEC_DE; // DEC $4000
        mem[1] = 0x20;
        mem[2] = 0x40;
        mem[0x4055] = 200;
        cpu.patch_memory(0, &mem);

        for _ in 1..512 {
            cpu.pc = 0x0000;
            cpu.step();
            expected = expected.wrapping_sub(1);

            assert_eq!(cpu.memory[0x4055], expected);

            let expected_z: bool = expected == 0;
            let expected_n = (expected & 0x80) > 0;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative > 0, expected_n);
        }
    }

    #[test]
    fn test_dec_c6() { // DEC $nn
        let mut cpu = Cpu::new();
        let mut expected: u8 = 200u8;

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = DEC_C6; // DEC $4000
        mem[1] = 0x20;
        mem[0x0020] = 200;
        cpu.patch_memory(0, &mem);

        for _ in 1..512 {
            cpu.pc = 0x0000;
            cpu.step();
            expected = expected.wrapping_sub(1);

            assert_eq!(cpu.memory[0x0020], expected);

            let expected_z: bool = expected == 0;
            let expected_n = (expected & 0x80) > 0;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative > 0, expected_n);
        }
    }

    #[test]
    fn test_dec_d6() { // DEC $nn,X
        let mut cpu = Cpu::new();
        cpu.x = 0x35;
        let mut expected: u8 = 200u8;

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = DEC_D6; // DEC $4000
        mem[1] = 0x20;
        mem[0x0055] = 200;
        cpu.patch_memory(0, &mem);

        for _ in 1..512 {
            cpu.pc = 0x0000;
            cpu.step();
            expected = expected.wrapping_sub(1);

            assert_eq!(cpu.memory[0x0055], expected);

            let expected_z: bool = expected == 0;
            let expected_n = (expected & 0x80) > 0;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative > 0, expected_n);
        }
    }

    #[test]
    fn test_dex() {
        let mut cpu = Cpu::new();
        let mem: [u8; 512] = [DEX_CA; 512];
        cpu.patch_memory(0, &mem);
        cpu.x = 100;
        let mut expected_x: i16 = 100;

        assert_eq!(cpu.x, expected_x as u8);

        cpu.step(); // DEX
        expected_x -= 1;

        assert_eq!(cpu.x, expected_x as u8);
        assert_eq!(cpu.p, 0);

        for _ in 1..512 {
            cpu.step(); // DEX
            expected_x -= 1;

            assert_eq!(cpu.x, expected_x as u8);

            let expected_z: bool = if (expected_x as u8) == 0 { true } else { false };
            let expected_n = expected_x as u8 & 0x80 > 0;
            assert_eq!(cpu.is_zero(), expected_z);
            assert_eq!(cpu.is_negative(), expected_n);
        }
    }

    #[test]
    fn test_dey() {
        let mut cpu = Cpu::new();
        let mem: [u8; 512] = [DEY_88; 512];
        cpu.patch_memory(0, &mem);
        cpu.y = 100;
        let mut expected_y: i16 = 100;

        assert_eq!(cpu.y, expected_y as u8);

        cpu.step(); // DEY
        expected_y -= 1;

        assert_eq!(cpu.y, expected_y as u8);
        assert_eq!(cpu.p, 0);

        for _ in 1..512 {
            cpu.step(); // DEY
            expected_y -= 1;

            assert_eq!(cpu.y, expected_y as u8);

            let expected_z: bool = if (expected_y as u8) == 0 { true } else { false };
            let expected_n = expected_y as u8 & 0x80 > 0;
            assert_eq!(cpu.is_zero(), expected_z);
            assert_eq!(cpu.is_negative(), expected_n);
        }
    }

    //
    // FLAGS
    //
    #[test]
    fn test_clc() {
        // clear carry flag
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[CLC_18]);
        cpu.p = Flags::C_Carry;
        assert!(cpu.is_carry());

        cpu.step(); // CLC
        assert!(!cpu.is_carry());
    }

    #[test]
    fn test_cld() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[CLD_D8]);
        cpu.update_decimal(true);
        assert!(cpu.is_decimal());

        cpu.step(); // CLD
        assert!(!cpu.is_decimal());
    }

    #[test]
    fn test_cli() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[CLI_58]);
        cpu.update_interrupt_disable(true);
        assert!(cpu.is_interrupt_disabled());

        cpu.step(); // CLI
        assert!(!cpu.is_interrupt_disabled());
    }

    #[test]
    fn test_clv() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[CLV_B8]);
        cpu.update_overflow(true);
        assert!(cpu.is_overflow());

        cpu.step(); // CLV
        assert!(!cpu.is_overflow());
    }

    #[test]
    fn test_sec() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[SEC_38]);
        assert!(!cpu.is_carry());

        cpu.step(); // SEC
        assert!(cpu.is_carry());
    }

    #[test]
    fn test_sed() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[SED_F8]);
        assert!(!cpu.is_decimal());

        cpu.step(); // SED
        assert!(cpu.is_decimal());
    }

    #[test]
    fn test_sei() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[SEI_78]);
        assert!(!cpu.is_interrupt_disabled());

        cpu.step(); // SEI
        assert!(cpu.is_interrupt_disabled());
    }

    //
    // TRANSFER
    //
    #[test]
    fn test_tax() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[TAX_AA, TAX_AA, TAX_AA]);
        cpu.a = 55;

        assert!(cpu.a == 55);
        assert!(cpu.x == 0);

        cpu.step(); // TAX
        assert!(cpu.a == 55);
        assert!(cpu.x == 55);
        assert!(!cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.a = 200;
        assert!(cpu.a == 200);
        assert!(cpu.x == 55);

        cpu.step(); // TAX
        assert!(cpu.a == 200);
        assert!(cpu.x == 200);
        assert!(cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.a = 0;
        assert!(cpu.a == 0);
        assert!(cpu.x == 200);

        cpu.step(); // TAX
        assert!(cpu.a == 0);
        assert!(cpu.x == 0);
        assert!(!cpu.is_negative());
        assert!(cpu.is_zero());
    }

    #[test]
    fn test_tay() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[TAY_A8, TAY_A8, TAY_A8]);
        cpu.a = 55;

        assert!(cpu.a == 55);
        assert!(cpu.y == 0);

        cpu.step(); // TAY
        assert!(cpu.a == 55);
        assert!(cpu.y == 55);
        assert!(!cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.a = 200;
        assert!(cpu.a == 200);
        assert!(cpu.y == 55);

        cpu.step(); // TAY
        assert!(cpu.a == 200);
        assert!(cpu.y == 200);
        assert!(cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.a = 0;
        assert!(cpu.a == 0);
        assert!(cpu.y == 200);

        cpu.step(); // TAY
        assert!(cpu.a == 0);
        assert!(cpu.y == 0);
        assert!(!cpu.is_negative());
        assert!(cpu.is_zero());
    }

    #[test]
    fn test_tsx() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[TSX_BA, TSX_BA, TSX_BA]);
        cpu.s = 55;

        assert!(cpu.s == 55);
        assert!(cpu.x == 0);

        cpu.step(); // TSX
        assert!(cpu.s == 55);
        assert!(cpu.x == 55);
        assert!(!cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.s = 200;
        assert!(cpu.s == 200);
        assert!(cpu.x == 55);

        cpu.step(); // TSX
        assert!(cpu.s == 200);
        assert!(cpu.x == 200);
        assert!(cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.s = 0;
        assert!(cpu.s == 0);
        assert!(cpu.x == 200);

        cpu.step(); // TSX
        assert!(cpu.s == 0);
        assert!(cpu.x == 0);
        assert!(!cpu.is_negative());
        assert!(cpu.is_zero());
    }

    #[test]
    fn test_txa() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[TXA_8A, TXA_8A, TXA_8A]);
        cpu.x = 55;

        assert!(cpu.x == 55);
        assert!(cpu.a == 0);

        cpu.step(); // TXA
        assert!(cpu.x == 55);
        assert!(cpu.a == 55);
        assert!(!cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.x = 200;
        assert!(cpu.x == 200);
        assert!(cpu.a == 55);

        cpu.step(); // TXA
        assert!(cpu.x == 200);
        assert!(cpu.a == 200);
        assert!(cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.x = 0;
        assert!(cpu.x == 0);
        assert!(cpu.a == 200);

        cpu.step(); // TXA
        assert!(cpu.x == 0);
        assert!(cpu.a == 0);
        assert!(!cpu.is_negative());
        assert!(cpu.is_zero());
    }

    #[test]
    fn test_txs() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[TXS_9A, TXS_9A, TXS_9A]);
        cpu.x = 55;

        assert!(cpu.x == 55);
        assert!(cpu.s == 0xff);

        cpu.step(); // TXS
        assert!(cpu.x == 55);
        assert!(cpu.s == 55);

        // --------------------------------------

        cpu.x = 200;
        assert!(cpu.x == 200);
        assert!(cpu.s == 55);

        cpu.step(); // TXS
        assert!(cpu.x == 200);
        assert!(cpu.s == 200);

        // --------------------------------------

        cpu.x = 0;
        assert!(cpu.x == 0);
        assert!(cpu.s == 200);

        cpu.step(); // TXS
        assert!(cpu.x == 0);
        assert!(cpu.s == 0);
    }

    #[test]
    fn test_tya() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[TYA_98, TYA_98, TYA_98]);
        cpu.y = 55;

        assert!(cpu.y == 55);
        assert!(cpu.a == 0);

        cpu.step(); // TYA
        assert!(cpu.y == 55);
        assert!(cpu.a == 55);
        assert!(!cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.y = 200;
        assert!(cpu.y == 200);
        assert!(cpu.a == 55);

        cpu.step(); // TYA
        assert!(cpu.y == 200);
        assert!(cpu.a == 200);
        assert!(cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.y = 0;
        assert!(cpu.y == 0);
        assert!(cpu.a == 200);

        cpu.step(); // TYA
        assert!(cpu.y == 0);
        assert!(cpu.a == 0);
        assert!(!cpu.is_negative());
        assert!(cpu.is_zero());
    }

    //
    // STACK
    //
    #[test]
    fn test_pha() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[PHA_48]);
        cpu.a = 0x28;

        assert!(cpu.s == 0xff);
        cpu.step(); // PHA

        assert!(cpu.s == 0xfe);
        assert!(cpu.a == 0x28);
        assert!(cpu.memory[cpu.s as usize + 1 + 0x0100] == 0x28)
    }

    #[test]
    fn test_php() {
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &[SEC_38, SED_F8, SEI_78, PHP_08]);

        cpu.step(); // SEC
        cpu.step(); // SED
        cpu.step(); // SEI

        assert!(cpu.is_carry());
        assert!(cpu.is_decimal());
        assert!(cpu.is_interrupt_disabled());

        cpu.step(); // PHP
        assert!(cpu.s == 0xfe);
        let pushed_value = cpu.memory[cpu.s as usize + 1 + 0x100];
        assert!(pushed_value & Flags::C_Carry != 0);
        assert!(pushed_value & Flags::D_Decimal != 0);
        assert!(pushed_value & Flags::I_InterruptDisable != 0);
    }

    #[test]
    fn test_pla() {
        let mut cpu = Cpu::new();
        cpu.a = 0x33;

        cpu.patch_memory(0, &[PHA_48, PLA_68]);
        cpu.step();
        cpu.a = 0x66;
        assert!(cpu.a == 0x66);
        assert!(cpu.s == 0xfe);

        cpu.step();
        assert!(cpu.a == 0x33);
        assert!(cpu.s == 0xff);
    }

    #[test]
    fn test_plp() {
        let mut cpu = Cpu::new();
        assert!(!cpu.is_carry());
        assert!(!cpu.is_decimal());
        assert!(!cpu.is_overflow());
        let saved_flags: u8 = Flags::C_Carry | Flags::D_Decimal | Flags::V_Overflow;
        cpu.a = saved_flags;

        cpu.patch_memory(0, &[PHA_48, PLP_28]);
        cpu.step();
        assert!(cpu.s == 0xfe);

        cpu.step();
        assert!(cpu.s == 0xff);
        assert!(cpu.is_carry());
        assert!(cpu.is_decimal());
        assert!(cpu.is_overflow());
    }

    //
    // CONTROL
    //
    /*
    #[test]
    fn test_brk_00() { // BRK
        panic!();
    }
    */

    #[test]
    fn test_jmp_4c() { // JMP $nnnn
        let mut cpu = Cpu::new();
        let mem = &[JMP_4C, 0x99, 0x70];

        cpu.patch_memory(0, mem);
        cpu.step();

        assert!(cpu.pc == 0x7099);
    }

    #[test]
    fn test_jmp_6c() { // JMP ($nnnn)
        let mut cpu = Cpu::new();
        let mem = &[JMP_6C, 0x00, 0x40];
        let mem2 = &[0x99, 0x70];

        cpu.patch_memory(0, mem);
        cpu.patch_memory(0x4000, mem2);
        cpu.step();

        assert!(cpu.pc == 0x7099);
    }

    #[test]
    fn test_jsr_20() { // JSR $nnnn
        let mut cpu = Cpu::new();
        let mem = &[JSR_20, 0x00, 0x80];

        cpu.patch_memory(0, mem);
        cpu.step();

        assert!(cpu.pc == 0x8000);
        assert!(cpu.s == (0xff - 2));
        assert!(cpu.memory[0x0100 + cpu.s as usize + 1] == 0x02); // JSR $8000 is at addr 0x0000,
        assert!(cpu.memory[0x0100 + cpu.s as usize + 2] == 0x00); // last byte of this instruction
                                                                  // is at 0x02;
    }

    #[test]
    fn test_rti_40() { // RTI
/*
        // FIXME: don't know how to write a proper test here
        //        what is the source of interrupt ??
        let mut cpu = Cpu::new();
        let mem_0x0000 = &[JSR_20, 0x00, 0x80,
                           TAY_A8,
        ];
        let mem_0x8000 = &[LDA_A9, 100,
                           ADC_69, 20,
                           RTS_60
        ];

        cpu.patch_memory(0, mem_0x0000);
        cpu.patch_memory(0x8000, mem_0x8000);
        cpu.step(); // JSR

        assert!(cpu.pc == 0x8000);
        assert!(cpu.s == (0xff - 2));
        assert!(cpu.memory[0x0100 + cpu.s as usize + 1] == 0x02);
        assert!(cpu.memory[0x0100 + cpu.s as usize + 2] == 0x00);

        cpu.step(); // LDA
        assert!(cpu.a == 100);

        cpu.step(); // ADC
        assert!(cpu.a == 120);

        cpu._dump_memory();
        println!(".1 cpu.pc: 0x{:04x}", cpu.pc);
        println!(".1 cpu.s: 0x{:02x}", cpu.s);
        cpu.step(); // RTS
        println!(".2 cpu.pc: 0x{:04x}", cpu.pc);
        println!(".2 cpu.s: 0x{:02x}", cpu.s);
        assert!(cpu.pc == 0x03);
        assert!(cpu.s == 0xff);

        cpu.step(); // TAY
        assert!(cpu.y == 120);
*/
    }

    #[test]
    fn test_rts_60() { // RTS
        let mut cpu = Cpu::new();
        let mem_0x0000 = &[JSR_20, 0x00, 0x80,
                           TAY_A8,
        ];
        let mem_0x8000 = &[LDA_A9, 100,
                           ADC_69, 20,
                           RTS_60
        ];

        cpu.patch_memory(0, mem_0x0000);
        cpu.patch_memory(0x8000, mem_0x8000);
        cpu.step(); // JSR

        assert!(cpu.pc == 0x8000);
        assert!(cpu.s == (0xff - 2));
        assert!(cpu.memory[0x0100 + cpu.s as usize + 1] == 0x02);
        assert!(cpu.memory[0x0100 + cpu.s as usize + 2] == 0x00);

        cpu.step(); // LDA
        assert!(cpu.a == 100);

        cpu.step(); // ADC
        assert!(cpu.a == 120);

        cpu._dump_memory();
        println!(".1 cpu.pc: 0x{:04x}", cpu.pc);
        println!(".1 cpu.s: 0x{:02x}", cpu.s);
        cpu.step(); // RTS
        println!(".2 cpu.pc: 0x{:04x}", cpu.pc);
        println!(".2 cpu.s: 0x{:02x}", cpu.s);
        assert!(cpu.pc == 0x03);
        assert!(cpu.s == 0xff);

        cpu.step(); // TAY
        assert!(cpu.y == 120);
    }

    //
    // BRANCH
    //
    #[test]
    fn test_bcc() {
        let memory: [u8; 3] = [BCC_90, 0x28, 0xA0]; // BCC 0xA028

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.step();
        assert!(cpu.pc == 0xA028);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.p |= Flags::C_Carry;
        cpu.step();
        assert!(cpu.pc == 0x0003);
    }

    #[test]
    fn test_bcs() {
        let memory: [u8; 3] = [BCS_B0, 0xF0, 0x80]; // BCS 0x80F0

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.step();
        assert!(cpu.pc == 0x0003);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.p |= Flags::C_Carry;
        cpu.step();
        assert!(cpu.pc == 0x80F0);
    }

    #[test]
    fn test_beq() {
        let memory: [u8; 3] = [BEQ_F0, 0xAA, 0x80];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.step();
        assert!(cpu.pc == 0x0003);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.p |= Flags::Z_Zero;
        cpu.step();
        assert!(cpu.pc == 0x80AA);
    }

    #[test]
    fn test_bmi() {
        let memory: [u8; 3] = [BMI_30, 0x00, 0x80];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.step();
        assert!(cpu.pc == 0x0003);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.p |= Flags::N_Negative;
        cpu.step();
        assert!(cpu.pc == 0x8000);
    }

    #[test]
    fn test_bne() {
        let memory: [u8; 3] = [BNE_D0, 0x00, 0x80];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.p |= Flags::Z_Zero;
        cpu.step();
        assert!(cpu.pc == 0x0003);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.step();
        assert!(cpu.pc == 0x8000);
    }

    #[test]
    fn test_bpl() {
        let memory: [u8; 3] = [BPL_10, 0x00, 0x40];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.p |= Flags::N_Negative;
        cpu.step();
        assert!(cpu.pc == 0x0003);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.step();
        assert!(cpu.pc == 0x4000);
    }

    #[test]
    fn test_bvc() {
        let memory: [u8; 3] = [BVC_50, 0x00, 0x40];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.p |= Flags::V_Overflow;
        cpu.step();
        assert!(cpu.pc == 0x0003);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.step();
        assert!(cpu.pc == 0x4000);
    }

    #[test]
    fn test_bvs() {
        let memory: [u8; 3] = [BVS_70, 0x00, 0x40];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.step();
        assert!(cpu.pc == 0x0003);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.p |= Flags::V_Overflow;
        cpu.step();
        assert!(cpu.pc == 0x4000);
    }

    //
    // SHIFT ASL
    //
    #[test]
    fn test_asl_0a() {
        let memory: &[u8] = &[ASL_0A];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.step();
        assert!(cpu.p == Z_Zero);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.a = 1;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.a == 2);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.a = 0x40;
        cpu.step();
        assert!(cpu.p == N_Negative);
        assert!(cpu.a == 0x80);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.a = 0x80;
        cpu.step();
        assert!(cpu.p == Z_Zero | C_Carry);
        assert!(cpu.a == 0x00);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.a = 0xC0;
        cpu.step();
        assert!(cpu.p == N_Negative | C_Carry);
        assert!(cpu.a == 0x80);
    }

    #[test]
    fn test_asl_0e() {
        let memory: &[u8] = &[ASL_0E, 0x00, 0x80];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8000] = 0;
        cpu.step();
        assert!(cpu.p == Z_Zero);
        assert!(cpu.memory[0x8000] == 0x00);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8000] = 1;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x8000] == 2);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8000] = 0x40;
        cpu.step();
        assert!(cpu.p == N_Negative);
        assert!(cpu.memory[0x8000] == 0x80);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8000] = 0x80;
        cpu.step();
        assert!(cpu.p == Z_Zero | C_Carry);
        assert!(cpu.memory[0x8000] == 0x00);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8000] = 0xC0;
        cpu.step();
        assert!(cpu.p == N_Negative | C_Carry);
        assert!(cpu.memory[0x8000] == 0x80);
    }

    #[test]
    fn test_asl_1e() {
        let memory: &[u8] = &[ASL_1E, 0x00, 0x80]; // addr: 0x8000 + X: 0x15

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8015] = 0;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == Z_Zero);
        assert!(cpu.memory[0x8015] == 0x00);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8015] = 1;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x8015] == 2);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8015] = 0x40;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == N_Negative);
        assert!(cpu.memory[0x8015] == 0x80);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8015] = 0x80;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == Z_Zero | C_Carry);
        assert!(cpu.memory[0x8015] == 0x00);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8015] = 0xC0;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == N_Negative | C_Carry);
        assert!(cpu.memory[0x8015] == 0x80);
    }

    #[test]
    fn test_asl_06() {
        let memory: &[u8] = &[ASL_06, 0x40]; // addr 0x0040, zero page

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0040] = 0;
        cpu.step();
        assert!(cpu.p == Z_Zero);
        assert!(cpu.memory[0x0040] == 0x00);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0040] = 1;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x0040] == 2);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0040] = 0x40;
        cpu.step();
        assert!(cpu.p == N_Negative);
        assert!(cpu.memory[0x0040] == 0x80);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0040] = 0x80;
        cpu.step();
        assert!(cpu.p == Z_Zero | C_Carry);
        assert!(cpu.memory[0x0040] == 0x00);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0040] = 0xC0;
        cpu.step();
        assert!(cpu.p == N_Negative | C_Carry);
        assert!(cpu.memory[0x0040] == 0x80);
    }

    #[test]
    fn test_asl_16() {
        let memory: &[u8] = &[ASL_16, 0x60]; // addr: 0x0060 + X: 0x15

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0075] = 0;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == Z_Zero);
        assert!(cpu.memory[0x0075] == 0x00);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0075] = 1;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x0075] == 2);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0075] = 0x40;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == N_Negative);
        assert!(cpu.memory[0x0075] == 0x80);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0075] = 0x80;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == Z_Zero | C_Carry);
        assert!(cpu.memory[0x0075] == 0x00);

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0075] = 0xC0;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == N_Negative | C_Carry);
        assert!(cpu.memory[0x0075] == 0x80);
    }

    //
    // SHIFT LSR
    //
    #[test]
    fn test_lsr_4a() {
        // LSR A
        let memory: &[u8] = &[LSR_4A];

        fn _t(memory: &[u8], val: u8, expected_val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.a = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.a == expected_val);
        }

        // memory   val exp_val, exp_flags
        _t(memory, 0, 0, Z_Zero);
        _t(memory, 1, 0, C_Carry | Z_Zero);
        _t(memory, 2, 1, 0);
        _t(memory, 3, 1, C_Carry);
        _t(memory, 0x80, 0x40, 0);
        _t(memory, 0x81, 0x40, C_Carry);
        _t(memory, 0xC9, 0x64, C_Carry);
    }

    #[test]
    fn test_lsr_4e() {
        // LSR $nnnn
        let memory: &[u8] = &[LSR_4E, 0x00, 0x80];

        fn _t(memory: &[u8], addr: u16, val: u8, expected_val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.memory[addr as usize] = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize] == expected_val);
        }

        // memory    addr,  val exp_val, exp_flags
        _t(memory, 0x8000, 0, 0, Z_Zero);
        _t(memory, 0x8000, 1, 0, C_Carry | Z_Zero);
        _t(memory, 0x8000, 2, 1, 0);
        _t(memory, 0x8000, 3, 1, C_Carry);
        _t(memory, 0x8000, 0x80, 0x40, 0);
        _t(memory, 0x8000, 0x81, 0x40, C_Carry);
        _t(memory, 0x8000, 0xC9, 0x64, C_Carry);
    }

    #[test]
    fn test_lsr_5e() {
        // LSR $nnnn,X
        let memory: &[u8] = &[LSR_5E, 0x00, 0x80];

        fn _t(memory: &[u8], addr: u16, x: u8, val: u8, expected_val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.x = x;
            cpu.memory[addr as usize + x as usize] = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize + x as usize] == expected_val);
        }

        // memory    addr,    x,  val exp_val, exp_flags
        _t(memory, 0x8000, 0x20, 0, 0, Z_Zero);
        _t(memory, 0x8000, 0x20, 1, 0, C_Carry | Z_Zero);
        _t(memory, 0x8000, 0x20, 2, 1, 0);
        _t(memory, 0x8000, 0x20, 3, 1, C_Carry);
        _t(memory, 0x8000, 0x20, 0x80, 0x40, 0);
        _t(memory, 0x8000, 0x20, 0x81, 0x40, C_Carry);
        _t(memory, 0x8000, 0x20, 0xC9, 0x64, C_Carry);
    }

    #[test]
    fn test_lsr_46() {
        // LSR $nn
        let memory: &[u8] = &[LSR_46, 0x80];

        fn _t(memory: &[u8], addr: u16, val: u8, expected_val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.memory[addr as usize] = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize] == expected_val);
        }

        // memory    addr,  val exp_val, exp_flags
        _t(memory, 0x0080, 0, 0, Z_Zero);
        _t(memory, 0x0080, 1, 0, C_Carry | Z_Zero);
        _t(memory, 0x0080, 2, 1, 0);
        _t(memory, 0x0080, 3, 1, C_Carry);
        _t(memory, 0x0080, 0x80, 0x40, 0);
        _t(memory, 0x0080, 0x81, 0x40, C_Carry);
        _t(memory, 0x0080, 0xC9, 0x64, C_Carry);
    }

    #[test]
    fn test_lsr_56() {
        // LSR $nn,X
        let memory: &[u8] = &[LSR_56, 0x80];

        fn _t(memory: &[u8], addr: u16, x: u8, val: u8, expected_val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.x = x;
            cpu.memory[addr as usize + x as usize] = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize + x as usize] == expected_val);
        }

        // memory    addr,    x,  val exp_val, exp_flags
        _t(memory, 0x0080, 0x20, 0, 0, Z_Zero);
        _t(memory, 0x0080, 0x20, 1, 0, C_Carry | Z_Zero);
        _t(memory, 0x0080, 0x20, 2, 1, 0);
        _t(memory, 0x0080, 0x20, 3, 1, C_Carry);
        _t(memory, 0x0080, 0x20, 0x80, 0x40, 0);
        _t(memory, 0x0080, 0x20, 0x81, 0x40, C_Carry);
        _t(memory, 0x0080, 0x20, 0xC9, 0x64, C_Carry);
    }

    // ROL - Rotate Left
    #[test]
    fn test_rol_2a() {
        let memory: &[u8] = &[ROL_2A]; // ROL A

        fn _t(memory: &[u8], set_carry: bool, val: u8, expected_val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.a = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.a == expected_val);
        }

        // memory  carry   val exp_val, exp_flags
        _t(memory, false, 0, 0, Z_Zero);
        _t(memory, false, 1, 2, 0);
        _t(memory, false, 0x40, 0x80, N_Negative);
        _t(memory, false, 0x80, 0, C_Carry | Z_Zero);
        _t(memory, false, 0xC0, 0x80, C_Carry | N_Negative);
        _t(memory, true, 0x40, 0x81, N_Negative);
        _t(memory, true, 0x80, 1, C_Carry);
        _t(memory, true, 0xC0, 0x81, N_Negative | C_Carry);
    }

    #[test]
    fn test_rol_2e() {
        let memory: &[u8] = &[ROL_2E, 0x00, 0x80]; // ROL $nnnn

        fn _t(
            memory: &[u8],
            addr: u16,
            set_carry: bool,
            val: u8,
            expected_val: u8,
            expected_flags: u8,
        ) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.memory[addr as usize] = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize] == expected_val);
        }

        // memory    addr  carry   val exp_val, exp_flags
        _t(memory, 0x8000, false, 0, 0, Z_Zero);
        _t(memory, 0x8000, false, 1, 2, 0);
        _t(memory, 0x8000, false, 0x40, 0x80, N_Negative);
        _t(memory, 0x8000, false, 0x80, 0, C_Carry | Z_Zero);
        _t(memory, 0x8000, false, 0xC0, 0x80, C_Carry | N_Negative);
        _t(memory, 0x8000, true, 0x40, 0x81, N_Negative);
        _t(memory, 0x8000, true, 0x80, 1, C_Carry);
        _t(memory, 0x8000, true, 0xC0, 0x81, N_Negative | C_Carry);
    }

    #[test]
    fn test_rol_3e() {
        let memory: &[u8] = &[ROL_3E, 0x00, 0x80]; // ROL $8000,X

        fn _t(
            memory: &[u8],
            addr: u16,
            x: u8,
            set_carry: bool,
            val: u8,
            expected_val: u8,
            expected_flags: u8,
        ) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.memory[addr as usize + x as usize] = val;
            cpu.x = x;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize + x as usize] == expected_val);
        }

        // memory    addr     x  carry  val  exp_val,  exp_flags
        _t(memory, 0x8000, 0x15, false, 0, 0, Z_Zero);
        _t(memory, 0x8000, 0x15, false, 1, 2, 0);
        _t(memory, 0x8000, 0x15, false, 0x40, 0x80, N_Negative);
        _t(memory, 0x8000, 0x15, false, 0x80, 0, C_Carry | Z_Zero);
        _t(
            memory,
            0x8000,
            0x15,
            false,
            0xC0,
            0x80,
            C_Carry | N_Negative,
        );
        _t(memory, 0x8000, 0x15, true, 0x40, 0x81, N_Negative);
        _t(memory, 0x8000, 0x15, true, 0x80, 1, C_Carry);
        _t(memory, 0x8000, 0x15, true, 0xC0, 0x81, N_Negative | C_Carry);
    }

    #[test]
    fn test_rol_26() {
        // ROL $nn
        let memory: &[u8] = &[ROL_26, 0x80]; // ROL $80

        fn _t(
            memory: &[u8],
            addr: u16,
            set_carry: bool,
            val: u8,
            expected_val: u8,
            expected_flags: u8,
        ) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.memory[addr as usize] = val;
            cpu.step();

            println!("cpu.p: {:02x}", cpu.p);
            println!("expected_flags: {:02x}", expected_flags);
            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize] == expected_val);
        }

        // memory    addr  carry  val exp_val,   exp_flags
        _t(memory, 0x0080, false, 0, 0, Z_Zero);
        _t(memory, 0x0080, false, 1, 2, 0);
        _t(memory, 0x0080, false, 0x40, 0x80, N_Negative);
        _t(memory, 0x0080, false, 0x80, 0, C_Carry | Z_Zero);
        _t(memory, 0x0080, false, 0xC0, 0x80, C_Carry | N_Negative);
        _t(memory, 0x0080, true, 0x40, 0x81, N_Negative);
        _t(memory, 0x0080, true, 0x80, 1, C_Carry);
        _t(memory, 0x0080, true, 0xC0, 0x81, N_Negative | C_Carry);
    }

    #[test]
    fn test_rol_36() {
        let memory: &[u8] = &[ROL_36, 0x80]; // ROL $80,X

        fn _t(
            memory: &[u8],
            addr: u16,
            x: u8,
            set_carry: bool,
            val: u8,
            expected_val: u8,
            expected_flags: u8,
        ) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.memory[addr as usize + x as usize] = val;
            cpu.x = x;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize + x as usize] == expected_val);
        }

        // memory    addr     x, carry  val exp_val,   exp_flags
        _t(memory, 0x0080, 0x15, false, 0, 0, Z_Zero);
        _t(memory, 0x0080, 0x15, false, 1, 2, 0);
        _t(memory, 0x0080, 0x15, false, 0x40, 0x80, N_Negative);
        _t(memory, 0x0080, 0x15, false, 0x80, 0, C_Carry | Z_Zero);
        _t(
            memory,
            0x0080,
            0x15,
            false,
            0xC0,
            0x80,
            C_Carry | N_Negative,
        );
        _t(memory, 0x0080, 0x15, true, 0x40, 0x81, N_Negative);
        _t(memory, 0x0080, 0x15, true, 0x80, 1, C_Carry);
        _t(memory, 0x0080, 0x15, true, 0xC0, 0x81, N_Negative | C_Carry);
    }

    // ROR - Rotate Right
    #[test]
    fn test_ror_6a() {
        let memory: &[u8] = &[ROR_6A]; // ROR A

        fn _t(memory: &[u8], set_carry: bool, val: u8, expected_val: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.a = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.a == expected_val);
        }

        // memory  carry  val  exp_val, exp_flags
        _t(memory, false, 0, 0, Z_Zero);
        _t(memory, false, 1, 0, C_Carry | Z_Zero);
        _t(memory, false, 2, 1, 0);
        _t(memory, false, 3, 1, C_Carry);
        _t(memory, true, 0, 0x80, N_Negative);
        _t(memory, true, 1, 0x80, N_Negative | C_Carry);
        _t(memory, true, 2, 0x81, N_Negative);
        _t(memory, true, 3, 0x81, N_Negative | C_Carry);
        _t(memory, true, 0x80, 0xC0, N_Negative);
        _t(memory, true, 0x81, 0xC0, N_Negative | C_Carry);
    }

    #[test]
    fn test_ror_6e() {
        let memory: &[u8] = &[ROR_6E, 0x40, 0x80]; // ROR $nnnn

        fn _t(
            memory: &[u8],
            addr: u16,
            set_carry: bool,
            val: u8,
            expected_val: u8,
            expected_flags: u8,
        ) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.memory[addr as usize] = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize] == expected_val);
        }

        // memory    addr, carry   val exp_val,  exp_flags
        _t(memory, 0x8040, false, 0, 0, Z_Zero);
        _t(memory, 0x8040, false, 1, 0, C_Carry | Z_Zero);
        _t(memory, 0x8040, false, 2, 1, 0);
        _t(memory, 0x8040, false, 3, 1, C_Carry);
        _t(memory, 0x8040, true, 0, 0x80, N_Negative);
        _t(memory, 0x8040, true, 1, 0x80, N_Negative | C_Carry);
        _t(memory, 0x8040, true, 2, 0x81, N_Negative);
        _t(memory, 0x8040, true, 3, 0x81, N_Negative | C_Carry);
        _t(memory, 0x8040, true, 0x80, 0xC0, N_Negative);
        _t(memory, 0x8040, true, 0x81, 0xC0, N_Negative | C_Carry);
    }

    #[test]
    fn test_ror_7e() {
        let memory: &[u8] = &[ROR_7E, 0x40, 0x80]; // ROR $nnnn,X

        fn _t(
            memory: &[u8],
            addr: u16,
            x: u8,
            set_carry: bool,
            val: u8,
            expected_val: u8,
            expected_flags: u8,
        ) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.x = x;
            cpu.update_carry(set_carry);
            cpu.memory[addr as usize + x as usize] = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize + x as usize] == expected_val);
        }

        // memory    addr,    x, carry   val exp_val,  exp_flags
        _t(memory, 0x8040, 0x20, false, 0, 0, Z_Zero);
        _t(memory, 0x8040, 0x20, false, 1, 0, C_Carry | Z_Zero);
        _t(memory, 0x8040, 0x20, false, 2, 1, 0);
        _t(memory, 0x8040, 0x20, false, 3, 1, C_Carry);
        _t(memory, 0x8040, 0x20, true, 0, 0x80, N_Negative);
        _t(memory, 0x8040, 0x20, true, 1, 0x80, N_Negative | C_Carry);
        _t(memory, 0x8040, 0x20, true, 2, 0x81, N_Negative);
        _t(memory, 0x8040, 0x20, true, 3, 0x81, N_Negative | C_Carry);
        _t(memory, 0x8040, 0x20, true, 0x80, 0xC0, N_Negative);
        _t(memory, 0x8040, 0x20, true, 0x81, 0xC0, N_Negative | C_Carry);
    }

    #[test]
    fn test_ror_66() {
        let memory: &[u8] = &[ROR_66, 0x40]; // ROR $nn

        fn _t(
            memory: &[u8],
            addr: u16,
            set_carry: bool,
            val: u8,
            expected_val: u8,
            expected_flags: u8,
        ) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.memory[addr as usize] = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize] == expected_val);
        }

        // memory    addr, carry   val exp_val,  exp_flags
        _t(memory, 0x0040, false, 0, 0, Z_Zero);
        _t(memory, 0x0040, false, 1, 0, C_Carry | Z_Zero);
        _t(memory, 0x0040, false, 2, 1, 0);
        _t(memory, 0x0040, false, 3, 1, C_Carry);
        _t(memory, 0x0040, true, 0, 0x80, N_Negative);
        _t(memory, 0x0040, true, 1, 0x80, N_Negative | C_Carry);
        _t(memory, 0x0040, true, 2, 0x81, N_Negative);
        _t(memory, 0x0040, true, 3, 0x81, N_Negative | C_Carry);
        _t(memory, 0x0040, true, 0x80, 0xC0, N_Negative);
        _t(memory, 0x0040, true, 0x81, 0xC0, N_Negative | C_Carry);
    }

    #[test]
    fn test_ror_76() {
        let memory: &[u8] = &[ROR_76, 0x40]; // ROR $nn,X

        fn _t(
            memory: &[u8],
            addr: u16,
            x: u8,
            set_carry: bool,
            val: u8,
            expected_val: u8,
            expected_flags: u8,
        ) {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.x = x;
            cpu.update_carry(set_carry);
            cpu.memory[addr as usize + x as usize] = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize + x as usize] == expected_val);
        }

        // memory    addr,    x, carry   val exp_val,  exp_flags
        _t(memory, 0x0040, 0x20, false, 0, 0, Z_Zero);
        _t(memory, 0x0040, 0x20, false, 1, 0, C_Carry | Z_Zero);
        _t(memory, 0x0040, 0x20, false, 2, 1, 0);
        _t(memory, 0x0040, 0x20, false, 3, 1, C_Carry);
        _t(memory, 0x0040, 0x20, true, 0, 0x80, N_Negative);
        _t(memory, 0x0040, 0x20, true, 1, 0x80, N_Negative | C_Carry);
        _t(memory, 0x0040, 0x20, true, 2, 0x81, N_Negative);
        _t(memory, 0x0040, 0x20, true, 3, 0x81, N_Negative | C_Carry);
        _t(memory, 0x0040, 0x20, true, 0x80, 0xC0, N_Negative);
        _t(memory, 0x0040, 0x20, true, 0x81, 0xC0, N_Negative | C_Carry);
    }

    //
    // AND
    //
    #[test]
    fn test_and_29() { // AND #$nn
        fn _t(a: u8, val: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            let memory: &[u8] = &[AND_29, val];
            cpu.patch_memory(0, memory);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //    a   val expect flags
        _t(   0,    0,    0, Z_Zero);
        _t(0xff, 0x55, 0x55, 0);
        _t(0xff, 0xaa, 0xaa, N_Negative);
        _t(0xaa, 0x55, 0x00, Z_Zero);
        _t(0xaa, 0xC0, 0x80, N_Negative);
        _t(0x3a, 0x7c, 0x38, 0);
    }

    #[test]
    fn test_and_2d() { // AND $nnnn
        let mut mem = [0u8; MEM_SZ];
        mem[0] = AND_2D; // AND $3320
        mem[1] = 0x20;
        mem[2] = 0x33;

        fn _t(mem: &[u8], a: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a expect flags
        mem[0x3320] = 0;
        _t(&mem,    0,    0, Z_Zero);
        mem[0x3320] = 0x55;
        _t(&mem, 0xff, 0x55, 0);
        mem[0x3320] = 0xaa;
        _t(&mem, 0xff, 0xaa, N_Negative);
        mem[0x3320] = 0;
        _t(&mem, 0xaa, 0x00, Z_Zero);
        mem[0x3320] = 0x80;
        _t(&mem, 0xaa, 0x80, N_Negative);
        mem[0x3320] = 0x7c;
        _t(&mem, 0x3a, 0x38, 0);
    }

    #[test]
    fn test_and_3d() { // AND $nnnn,X
        let mut mem = [0u8; MEM_SZ];
        mem[0] = AND_3D; // AND $3320,X
        mem[1] = 0x20;
        mem[2] = 0x33;

        fn _t(mem: &[u8], a: u8, x: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x3340] = 0;
        _t(&mem,    0, 0x20,    0, Z_Zero);
        mem[0x3340] = 0x55;
        _t(&mem, 0xff, 0x20, 0x55, 0);
        mem[0x3340] = 0xaa;
        _t(&mem, 0xff, 0x20, 0xaa, N_Negative);
        mem[0x3340] = 0;
        _t(&mem, 0xaa, 0x20, 0x00, Z_Zero);
        mem[0x3340] = 0x80;
        _t(&mem, 0xaa, 0x20, 0x80, N_Negative);
        mem[0x3340] = 0x7c;
        _t(&mem, 0x3a, 0x20, 0x38, 0);
    }

    #[test]
    fn test_and_39() { // AND $nnnn,Y
        let mut mem = [0u8; MEM_SZ];
        mem[0] = AND_39; // AND $3320,Y
        mem[1] = 0x20;
        mem[2] = 0x33;

        fn _t(mem: &[u8], a: u8, y: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x3340] = 0;
        _t(&mem,    0, 0x20,    0, Z_Zero);
        mem[0x3340] = 0x55;
        _t(&mem, 0xff, 0x20, 0x55, 0);
        mem[0x3340] = 0xaa;
        _t(&mem, 0xff, 0x20, 0xaa, N_Negative);
        mem[0x3340] = 0;
        _t(&mem, 0xaa, 0x20, 0x00, Z_Zero);
        mem[0x3340] = 0x80;
        _t(&mem, 0xaa, 0x20, 0x80, N_Negative);
        mem[0x3340] = 0x7c;
        _t(&mem, 0x3a, 0x20, 0x38, 0);
    }

    #[test]
    fn test_and_25() { // AND $nn
        let mut mem = [0u8; MEM_SZ];
        mem[0] = AND_25; // AND $33
        mem[1] = 0x33;

        fn _t(mem: &[u8], a: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a expect flags
        mem[0x0033] = 0;
        _t(&mem,    0,    0, Z_Zero);
        mem[0x0033] = 0x55;
        _t(&mem, 0xff, 0x55, 0);
        mem[0x0033] = 0xaa;
        _t(&mem, 0xff, 0xaa, N_Negative);
        mem[0x0033] = 0;
        _t(&mem, 0xaa, 0x00, Z_Zero);
        mem[0x0033] = 0x80;
        _t(&mem, 0xaa, 0x80, N_Negative);
        mem[0x0033] = 0x7c;
        _t(&mem, 0x3a, 0x38, 0);
    }

    #[test]
    fn test_and_35() { // AND $nn,X
        let mut mem = [0u8; MEM_SZ];
        mem[0] = AND_35; // AND $44,X
        mem[1] = 0x44;

        fn _t(mem: &[u8], a: u8, x: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x0055] = 0;
        _t(&mem,    0, 0x11,    0, Z_Zero);
        mem[0x0055] = 0x55;
        _t(&mem, 0xff, 0x11, 0x55, 0);
        mem[0x0055] = 0xaa;
        _t(&mem, 0xff, 0x11, 0xaa, N_Negative);
        mem[0x0055] = 0;
        _t(&mem, 0xaa, 0x11, 0x00, Z_Zero);
        mem[0x0055] = 0x80;
        _t(&mem, 0xaa, 0x11, 0x80, N_Negative);
        mem[0x0055] = 0x7c;
        _t(&mem, 0x3a, 0x11, 0x38, 0);
    }

    #[test]
    fn test_and_21() { // AND ($nn,X)
        let mut mem = [0u8; MEM_SZ];
        mem[0] = AND_21; // AND ($44,X)
        mem[1] = 0x44;
        mem[0x0055] = 0xf0; // 0x80f0
        mem[0x0056] = 0x80;

        fn _t(mem: &[u8], a: u8, x: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x80f0] = 0;
        _t(&mem,    0, 0x11,    0, Z_Zero);
        mem[0x80f0] = 0x55;
        _t(&mem, 0xff, 0x11, 0x55, 0);
        mem[0x80f0] = 0xaa;
        _t(&mem, 0xff, 0x11, 0xaa, N_Negative);
        mem[0x80f0] = 0;
        _t(&mem, 0xaa, 0x11, 0x00, Z_Zero);
        mem[0x80f0] = 0x80;
        _t(&mem, 0xaa, 0x11, 0x80, N_Negative);
        mem[0x80f0] = 0x7c;
        _t(&mem, 0x3a, 0x11, 0x38, 0);
    }

    #[test]
    fn test_and_31() { // AND ($nn),Y
        let mut mem = [0u8; MEM_SZ];
        mem[0] = AND_31; // AND ($44,Y)
        mem[1] = 0x44;
        mem[0x0044] = 0xf0; // 0x80f0
        mem[0x0045] = 0x80;

        fn _t(mem: &[u8], a: u8, y: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    y expect flags
        mem[0x8101] = 0;
        _t(&mem,    0, 0x11,    0, Z_Zero);
        mem[0x8101] = 0x55;
        _t(&mem, 0xff, 0x11, 0x55, 0);
        mem[0x8101] = 0xaa;
        _t(&mem, 0xff, 0x11, 0xaa, N_Negative);
        mem[0x8101] = 0;
        _t(&mem, 0xaa, 0x11, 0x00, Z_Zero);
        mem[0x8101] = 0x80;
        _t(&mem, 0xaa, 0x11, 0x80, N_Negative);
        mem[0x8101] = 0x7c;
        _t(&mem, 0x3a, 0x11, 0x38, 0);
    }

    //
    // BIT
    //
    #[test]
    fn test_bit_2c() { // BIT $nnnn
        let mut mem = [0u8; MEM_SZ];
        mem[0] = BIT_2C; // BIT $1000
        mem[1] = 0x00;
        mem[2] = 0x10;

        fn _t(mem: &[u8], a: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == a);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a flags
        mem[0x1000] = 0;
        _t(&mem,    0, Z_Zero);
        mem[0x1000] = 0x55;
        _t(&mem, 0xff, V_Overflow);
        mem[0x1000] = 0xaa;
        _t(&mem, 0xff, N_Negative);
        mem[0x1000] = 0;
        _t(&mem, 0xaa, Z_Zero);
        mem[0x1000] = 0x80;
        _t(&mem, 0xaa, N_Negative);
        mem[0x1000] = 0x7c;
        _t(&mem, 0x3a, 0);
    }

    #[test]
    fn test_bit_24() { // BIT $nn
        let mut mem = [0u8; MEM_SZ];
        mem[0] = BIT_2C; // BIT $90
        mem[1] = 0x90;

        fn _t(mem: &[u8], a: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == a);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a flags
        mem[0x0090] = 0;
        _t(&mem,    0, Z_Zero);
        mem[0x0090] = 0x55;
        _t(&mem, 0xaa, Z_Zero);
        mem[0x0090] = 0x55;
        _t(&mem, 0xff, V_Overflow);
        mem[0x0090] = 0xaa;
        _t(&mem, 0xff, N_Negative);
        mem[0x0090] = 0;
        _t(&mem, 0xaa, Z_Zero);
        mem[0x0090] = 0x80;
        _t(&mem, 0xaa, N_Negative);
        mem[0x0090] = 0x7c;
        _t(&mem, 0x3a, 0);
    }

    //
    // EOR
    //
    #[test]
    fn test_eor_49() { // EOR #$nn
        fn _t(a: u8, val: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            let memory: &[u8] = &[EOR_49, val];
            cpu.patch_memory(0, memory);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //    a   val expect flags
        _t(   0,    0,    0, Z_Zero);
        _t(0xff, 0x55, 0xaa, N_Negative);
        _t(0xff, 0xaa, 0x55, 0);
        _t(0xaa, 0x55, 0xff, N_Negative);
        _t(0xaa, 0xc0, 0x6a, 0);
        _t(0x3a, 0x7c, 0x46, 0);
    }

    #[test]
    fn test_eor_4d() { // EOR $nnnn
        let mut mem = [0u8; MEM_SZ];
        mem[0] = EOR_4D; // EOR $3320
        mem[1] = 0x20;
        mem[2] = 0x33;

        fn _t(mem: &[u8], a: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a expect flags
        mem[0x3320] = 0;
        _t(&mem,    0,    0, Z_Zero);
        mem[0x3320] = 0x55;
        _t(&mem, 0xff, 0xaa, N_Negative);
        mem[0x3320] = 0xaa;
        _t(&mem, 0xff, 0x55, 0);
        mem[0x3320] = 0;
        _t(&mem, 0xaa, 0xaa, N_Negative);
        mem[0x3320] = 0x80;
        _t(&mem, 0xaa, 0x2a, 0);
        mem[0x3320] = 0x7c;
        _t(&mem, 0x3a, 0x46, 0);
    }

    #[test]
    fn test_eor_5d() { // EOR $nnnn,X
        let mut mem = [0u8; MEM_SZ];
        mem[0] = EOR_5D; // EOR $3320,X
        mem[1] = 0x20;
        mem[2] = 0x33;

        fn _t(mem: &[u8], a: u8, x: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x3340] = 0;
        _t(&mem,    0, 0x20,    0, Z_Zero);
        mem[0x3340] = 0x55;
        _t(&mem, 0xff, 0x20, 0xaa, N_Negative);
        mem[0x3340] = 0xaa;
        _t(&mem, 0xff, 0x20, 0x55, 0);
        mem[0x3340] = 0;
        _t(&mem, 0xaa, 0x20, 0xaa, N_Negative);
        mem[0x3340] = 0x80;
        _t(&mem, 0xaa, 0x20, 0x2a, 0);
        mem[0x3340] = 0x7c;
        _t(&mem, 0x3a, 0x20, 0x46, 0);
    }

    #[test]
    fn test_eor_59() { // EOR $nnnn,Y
        let mut mem = [0u8; MEM_SZ];
        mem[0] = EOR_59; // EOR $3320,Y
        mem[1] = 0x20;
        mem[2] = 0x33;

        fn _t(mem: &[u8], a: u8, y: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x3340] = 0;
        _t(&mem,    0, 0x20,    0, Z_Zero);
        mem[0x3340] = 0x55;
        _t(&mem, 0xff, 0x20, 0xaa, N_Negative);
        mem[0x3340] = 0xaa;
        _t(&mem, 0xff, 0x20, 0x55, 0);
        mem[0x3340] = 0;
        _t(&mem, 0xaa, 0x20, 0xaa, N_Negative);
        mem[0x3340] = 0x80;
        _t(&mem, 0xaa, 0x20, 0x2a, 0);
        mem[0x3340] = 0x7c;
        _t(&mem, 0x3a, 0x20, 0x46, 0);
    }

    #[test]
    fn test_eor_45() { // EOR $nn
        let mut mem = [0u8; MEM_SZ];
        mem[0] = EOR_45; // EOR $33
        mem[1] = 0x33;

        fn _t(mem: &[u8], a: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a expect flags
        mem[0x0033] = 0;
        _t(&mem,    0,    0, Z_Zero);
        mem[0x0033] = 0x55;
        _t(&mem, 0xff, 0xaa, N_Negative);
        mem[0x0033] = 0xaa;
        _t(&mem, 0xff, 0x55, 0);
        mem[0x0033] = 0;
        _t(&mem, 0xaa, 0xaa, N_Negative);
        mem[0x0033] = 0x80;
        _t(&mem, 0xaa, 0x2a, 0);
        mem[0x0033] = 0x7c;
        _t(&mem, 0x3a, 0x46, 0);
    }

    #[test]
    fn test_eor_55() { // EOR $nn,X
        let mut mem = [0u8; MEM_SZ];
        mem[0] = EOR_55; // EOR $44,X
        mem[1] = 0x44;

        fn _t(mem: &[u8], a: u8, x: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x0055] = 0;
        _t(&mem,    0, 0x11,    0, Z_Zero);
        mem[0x0055] = 0x55;
        _t(&mem, 0xff, 0x11, 0xaa, N_Negative);
        mem[0x0055] = 0xaa;
        _t(&mem, 0xff, 0x11, 0x55, 0);
        mem[0x0055] = 0;
        _t(&mem, 0xaa, 0x11, 0xaa, N_Negative);
        mem[0x0055] = 0x80;
        _t(&mem, 0xaa, 0x11, 0x2a, 0);
        mem[0x0055] = 0x7c;
        _t(&mem, 0x3a, 0x11, 0x46, 0);
    }

    #[test]
    fn test_eor_41() { // EOR ($nn,X)
        let mut mem = [0u8; MEM_SZ];
        mem[0] = EOR_41; // EOR ($44,X)
        mem[1] = 0x44;
        mem[0x0055] = 0xf0; // 0x80f0
        mem[0x0056] = 0x80;

        fn _t(mem: &[u8], a: u8, x: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x80f0] = 0;
        _t(&mem,    0, 0x11,    0, Z_Zero);
        mem[0x80f0] = 0x55;
        _t(&mem, 0xff, 0x11, 0xaa, N_Negative);
        mem[0x80f0] = 0xaa;
        _t(&mem, 0xff, 0x11, 0x55, 0);
        mem[0x80f0] = 0;
        _t(&mem, 0xaa, 0x11, 0xaa, N_Negative);
        mem[0x80f0] = 0x80;
        _t(&mem, 0xaa, 0x11, 0x2a, 0);
        mem[0x80f0] = 0x7c;
        _t(&mem, 0x3a, 0x11, 0x46, 0);
    }

    #[test]
    fn test_eor_51() { // EOR ($nn),Y
        let mut mem = [0u8; MEM_SZ];
        mem[0] = EOR_51; // EOR ($44,Y)
        mem[1] = 0x44;
        mem[0x0044] = 0xf0; // 0x80f0
        mem[0x0045] = 0x80;

        fn _t(mem: &[u8], a: u8, y: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    y expect flags
        mem[0x8101] = 0;
        _t(&mem,    0, 0x11,    0, Z_Zero);
        mem[0x8101] = 0x55;
        _t(&mem, 0xff, 0x11, 0xaa, N_Negative);
        mem[0x8101] = 0xaa;
        _t(&mem, 0xff, 0x11, 0x55, 0);
        mem[0x8101] = 0;
        _t(&mem, 0xaa, 0x11, 0xaa, N_Negative);
        mem[0x8101] = 0x80;
        _t(&mem, 0xaa, 0x11, 0x2a, 0);
        mem[0x8101] = 0x7c;
        _t(&mem, 0x3a, 0x11, 0x46, 0);
    }

    //
    // ORA
    //
    #[test]
    fn test_ora_09() { // ORA #$nn
        fn _t(a: u8, val: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            let memory: &[u8] = &[ORA_09, val];
            cpu.patch_memory(0, memory);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //    a   val expect flags
        dbg!(".0");
        _t(   0,    0,    0, Z_Zero);
        dbg!(".1");
        _t(0xff, 0x55, 0xff, N_Negative);
        dbg!(".2");
        _t(0xff, 0xaa, 0xff, N_Negative);
        dbg!(".3");
        _t(0xaa, 0x55, 0xff, N_Negative);
        dbg!(".4");
        _t(0xaa, 0xc0, 0xea, N_Negative);
        dbg!(".5");
        _t(0x3a, 0x7c, 0x7e, 0);
    }

    #[test]
    fn test_ora_0d() { // ORA $nnnn
        let mut mem = [0u8; MEM_SZ];
        mem[0] = ORA_0D; // ORA $3320
        mem[1] = 0x20;
        mem[2] = 0x33;

        fn _t(mem: &[u8], a: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a expect flags
        mem[0x3320] = 0;
        _t(&mem,    0,    0, Z_Zero);
        mem[0x3320] = 0x55;
        _t(&mem, 0xff, 0xff, N_Negative);
        mem[0x3320] = 0xaa;
        _t(&mem, 0xff, 0xff, N_Negative);
        mem[0x3320] = 0;
        _t(&mem, 0xaa, 0xaa, N_Negative);
        mem[0x3320] = 0x80;
        _t(&mem, 0xaa, 0xaa, N_Negative);
        mem[0x3320] = 0x7c;
        _t(&mem, 0x3a, 0x7e, 0);
    }

    #[test]
    fn test_ora_1d() { // ORA $nnnn,X
        let mut mem = [0u8; MEM_SZ];
        mem[0] = ORA_1D; // ORA $3320,X
        mem[1] = 0x20;
        mem[2] = 0x33;

        fn _t(mem: &[u8], a: u8, x: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x3340] = 0;
        _t(&mem,    0, 0x20,    0, Z_Zero);
        mem[0x3340] = 0x55;
        _t(&mem, 0xff, 0x20, 0xff, N_Negative);
        mem[0x3340] = 0xaa;
        _t(&mem, 0xff, 0x20, 0xff, N_Negative);
        mem[0x3340] = 0;
        _t(&mem, 0xaa, 0x20, 0xaa, N_Negative);
        mem[0x3340] = 0x80;
        _t(&mem, 0xaa, 0x20, 0xaa, N_Negative);
        mem[0x3340] = 0x7c;
        _t(&mem, 0x3a, 0x20, 0x7e, 0);
    }

    #[test]
    fn test_ora_19() { // ORA $nnnn,Y
        let mut mem = [0u8; MEM_SZ];
        mem[0] = ORA_19; // ORA $3320,Y
        mem[1] = 0x20;
        mem[2] = 0x33;

        fn _t(mem: &[u8], a: u8, y: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x3340] = 0;
        _t(&mem,    0, 0x20,    0, Z_Zero);
        mem[0x3340] = 0x55;
        _t(&mem, 0xff, 0x20, 0xff, N_Negative);
        mem[0x3340] = 0xaa;
        _t(&mem, 0xff, 0x20, 0xff, N_Negative);
        mem[0x3340] = 0;
        _t(&mem, 0xaa, 0x20, 0xaa, N_Negative);
        mem[0x3340] = 0x80;
        _t(&mem, 0xaa, 0x20, 0xaa, N_Negative);
        mem[0x3340] = 0x7c;
        _t(&mem, 0x3a, 0x20, 0x7e, 0);
    }

    #[test]
    fn test_ora_05() { // ORA $nn
        let mut mem = [0u8; MEM_SZ];
        mem[0] = ORA_05; // ORA $33
        mem[1] = 0x33;

        fn _t(mem: &[u8], a: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a expect flags
        mem[0x0033] = 0;
        _t(&mem,    0,    0, Z_Zero);
        mem[0x0033] = 0x55;
        _t(&mem, 0xff, 0xff, N_Negative);
        mem[0x0033] = 0xaa;
        _t(&mem, 0xff, 0xff, N_Negative);
        mem[0x0033] = 0;
        _t(&mem, 0xaa, 0xaa, N_Negative);
        mem[0x0033] = 0x80;
        _t(&mem, 0xaa, 0xaa, N_Negative);
        mem[0x0033] = 0x7c;
        _t(&mem, 0x3a, 0x7e, 0);
    }

    #[test]
    fn test_ora_15() { // ORA $nn,X
        let mut mem = [0u8; MEM_SZ];
        mem[0] = ORA_15; // ORA $44,X
        mem[1] = 0x44;

        fn _t(mem: &[u8], a: u8, x: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x0055] = 0;
        _t(&mem,    0, 0x11,    0, Z_Zero);
        mem[0x0055] = 0x55;
        _t(&mem, 0xff, 0x11, 0xff, N_Negative);
        mem[0x0055] = 0xaa;
        _t(&mem, 0xff, 0x11, 0xff, N_Negative);
        mem[0x0055] = 0;
        _t(&mem, 0xaa, 0x11, 0xaa, N_Negative);
        mem[0x0055] = 0x80;
        _t(&mem, 0xaa, 0x11, 0xaa, N_Negative);
        mem[0x0055] = 0x7c;
        _t(&mem, 0x3a, 0x11, 0x7e, 0);
    }

    #[test]
    fn test_ora_01() { // ORA ($nn,X)
        let mut mem = [0u8; MEM_SZ];
        mem[0] = ORA_01; // ORA ($44,X)
        mem[1] = 0x44;
        mem[0x0055] = 0xf0; // 0x80f0
        mem[0x0056] = 0x80;

        fn _t(mem: &[u8], a: u8, x: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.x = x;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    x expect flags
        mem[0x80f0] = 0;
        _t(&mem,    0, 0x11,    0, Z_Zero);
        mem[0x80f0] = 0x55;
        _t(&mem, 0xff, 0x11, 0xff, N_Negative);
        mem[0x80f0] = 0xaa;
        _t(&mem, 0xff, 0x11, 0xff, N_Negative);
        mem[0x80f0] = 0;
        _t(&mem, 0xaa, 0x11, 0xaa, N_Negative);
        mem[0x80f0] = 0x80;
        _t(&mem, 0xaa, 0x11, 0xaa, N_Negative);
        mem[0x80f0] = 0x7c;
        _t(&mem, 0x3a, 0x11, 0x7e, 0);
    }

    #[test]
    fn test_ora_11() { // ORA ($nn),Y
        let mut mem = [0u8; MEM_SZ];
        mem[0] = ORA_11; // ORA ($44,Y)
        mem[1] = 0x44;
        mem[0x0044] = 0xf0; // 0x80f0
        mem[0x0045] = 0x80;

        fn _t(mem: &[u8], a: u8, y: u8, expect: u8, expected_flags: u8) {
            let mut cpu = Cpu::new();
            cpu.a = a;
            cpu.y = y;
            cpu.patch_memory(0, mem);
            cpu.step();

            assert!(cpu.a == expect);
            assert!(cpu.p == expected_flags);
        }

        //  mem     a    y expect flags
        mem[0x8101] = 0;
        _t(&mem,    0, 0x11,    0, Z_Zero);
        mem[0x8101] = 0x55;
        _t(&mem, 0xff, 0x11, 0xff, N_Negative);
        mem[0x8101] = 0xaa;
        _t(&mem, 0xff, 0x11, 0xff, N_Negative);
        mem[0x8101] = 0;
        _t(&mem, 0xaa, 0x11, 0xaa, N_Negative);
        mem[0x8101] = 0x80;
        _t(&mem, 0xaa, 0x11, 0xaa, N_Negative);
        mem[0x8101] = 0x7c;
        _t(&mem, 0x3a, 0x11, 0x7e, 0);
    }
}
