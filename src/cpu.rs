// https://www.pagetable.com/c64ref/6502/?tab=2
// http://www.emulator101.com/6502-addressing-modes.html
// https://web.archive.org/web/20221112230813if_/http://archive.6502.org/books/mcs6500_family_programming_manual.pdf

// TODO: Make Cpu reference to memory, not owning it
//       (it may speed up tests a little bit)

// TODO: Make all the tests look the same: all have some _t function
//       All have ready memory, no memory modification during tests,
//          no pre-calculated addresses passed to test

// TODO: Add separate functions for different addressing modes
// TODO: use update_*() all over the code
// TODO: update tests for AND, ORA, EOR to have common code for different addressing modes
// TODO: Crossing page boundary (page zero addressing) ? What does it mean ?
// TODO: _inc_inst
// TODO: tests: make the cpu to start execution from some `org`, not from 0x00.
//       it will allow to test the zero_page overlap (addresses 0xff and 0x00).
// TODO: make all memory addressing aux functions return not the reference but address (usize)
// TODO: add aux function to work with stack (push, push_2b, pop, pop_2b)
// TODO: add tests for CONTROL instructions
// TODO: run test roms for 6502

const MEM_SZ: usize = 65_536;

#[derive(Debug, PartialEq)]
pub(crate) struct Cpu {
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

// FIXME: remove this
#[allow(dead_code)]
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

// FIXME:
#[allow(dead_code)]
impl Cpu {
    fn new() -> Cpu {
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

    fn set_pc_to_current_addr_in_memory(&mut self) {
        self.pc = self.get_addr() as u16;
    }

    // in the ideal world get_addr and get_addr_zero_page would return u16
    // as the address, cause we have only 64K of RAM
    //
    // but usize is more appropriate cause it makes me do less `as usize`
    // conversions to point inside rust [u8; ...] array
    fn get_addr(&mut self) -> usize {
        let mut addr = self.memory[self.pc as usize] as usize;
        addr |= (self.memory[self.pc as usize + 1] as usize) << 8;
        addr
    }

    fn get_addr_zero_page(&mut self) -> usize {
        self.memory[self.pc as usize] as usize
    }

    fn _asl_inst(&mut self, mut val: u8) -> u8 {
        self.update_carry(val & 0x80 != 0);
        val <<= 1;
        self.update_negative(val & 0x80 != 0);
        self.update_zero(val == 0);
        val
    }

    fn _lsr_inst(&mut self, mut val: u8) -> u8 {
        self.update_carry(val & 1 != 0);
        val >>= 1;
        self.update_negative(false);
        self.update_zero(val == 0);
        val
    }

    fn _rol_inst(&mut self, mut val: u8) -> u8 {
        let prev_carry = self.is_carry();
        self.update_carry(val & 0x80 != 0);
        val <<= 1;
        if prev_carry {
            val |= 1;
        }
        self.update_negative(val & 0x80 != 0);
        self.update_zero(val == 0);
        val
    }

    fn _ror_inst(&mut self, mut val: u8) -> u8 {
        let prev_carry = self.is_carry();
        self.update_carry(val & 1 != 0);
        val >>= 1;
        if prev_carry {
            val |= 0x80;
        }
        self.update_negative(val & 0x80 != 0);
        self.update_zero(val == 0);
        val
    }

    fn _and_inst(&mut self, val: u8) {
        self.a &= val;
        self.update_zero(self.a == 0);
        self.update_negative(self.a & 0x80 != 0);
    }

    fn _bit_inst(&mut self, val: u8) {
        let r = self.a & val;
        self.update_zero(r == 0);
        self.update_overflow(r & 0x40 != 0);
        self.update_negative(r & 0x80 != 0);
    }

    fn _eor_inst(&mut self, val: u8) {
        self.a ^= val;
        self.update_zero(self.a == 0);
        self.update_negative(self.a & 0x80 != 0);
    }

    fn _ora_inst(&mut self, val: u8) {
        self.a |= val;
        self.update_zero(self.a == 0);
        self.update_negative(self.a & 0x80 != 0);
    }

    //
    // Addressing
    //
    fn _immediate(&mut self) -> u8 {
        self.memory[self.pc as usize]
    }

    fn _absolute(&mut self) -> &mut u8 {
        let addr = self.get_addr();
        &mut self.memory[addr]
    }

    fn _absolute_addr(&mut self) -> usize {
        let addr = self.get_addr();
        addr
    }

    fn _absolute_indirect_addr(&mut self) -> usize {
        let addr = self.get_addr();
        let mut addr2 = self.memory[addr] as usize;
        addr2 += (self.memory[addr+1] as usize) << 8;
        addr2
    }

    fn _absolute_x(&mut self) -> &mut u8 {
        let addr = self.get_addr() + self.x as usize;
        &mut self.memory[addr]
    }

    fn _absolute_y(&mut self) -> &mut u8 {
        let addr = self.get_addr() + self.y as usize;
        &mut self.memory[addr]
    }

    fn _zero_page(&mut self) -> &mut u8 {
        let addr = self.get_addr_zero_page();
        &mut self.memory[addr]
    }

    fn _zero_page_x(&mut self) -> &mut u8 {
        let addr = (self.get_addr_zero_page() + self.x as usize) & 0xff;
        &mut self.memory[addr]
    }

    fn _zero_page_y(&mut self) -> &mut u8 {
        let addr = (self.get_addr_zero_page() + self.y as usize) & 0xff;
        &mut self.memory[addr]
    }

    fn _zero_page_x_indirect(&mut self) -> &mut u8 {
        let addr = (self.get_addr_zero_page() + self.x as usize) & 0xff;
        let mut addr2 = self.memory[addr] as usize;
        addr2 += (self.memory[(addr + 1) & 0xff] as usize) << 8;
        &mut self.memory[addr2]
    }

    fn _zero_page_indirect_y(&mut self) -> &mut u8 {
        let addr = self.get_addr_zero_page();
        let mut addr2 = self.memory[addr] as usize; // lo
        addr2 += (self.memory[(addr+1) & 0xff] as usize) << 8; // hi
        addr2 += self.y as usize;
        &mut self.memory[addr2]
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
            opcodes::LDA_A9 => { // LDA #$nn
                self.a = self._immediate();
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
                self.pc += 1;
            }
            opcodes::LDA_AD => { // LDA $nnnn
                self.a = *self._absolute();
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
                self.pc += 2;
            }
            opcodes::LDA_BD => { // LDA $nnnn,x
                self.a = *self._absolute_x();
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
                self.pc += 2;
            }
            opcodes::LDA_B9 => { // LDA $nnnn,y
                self.a = *self._absolute_y();
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
                self.pc += 2;
            }
            opcodes::LDA_A5 => { // LDA $nn
                self.a = *self._zero_page();
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
                self.pc += 1;
            }
            opcodes::LDA_B5 => { // LDA $nn,X
                self.a = *self._zero_page_x();
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
                self.pc += 1;
            }
            opcodes::LDA_A1 => { // LDA ($nn,X)
                self.a = *self._zero_page_x_indirect();
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
                self.pc += 1;
            }
            opcodes::LDA_B1 => { // LDA ($nn),Y
                self.a = *self._zero_page_indirect_y();
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
                self.pc += 1;
            }

            //
            // LOAD - LDX
            //
            opcodes::LDX_A2 => { // LDX #$nn
                self.x = self._immediate();
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
                self.pc += 1;
            }
            opcodes::LDX_AE => { // LDX $nnnn
                self.x = *self._absolute();
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
                self.pc += 2;
            }
            opcodes::LDX_BE => { // LDX $nnnn,Y
                self.x = *self._absolute_y();
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
                self.pc += 2;
            }
            opcodes::LDX_A6 => { // LDX $nn
                self.x = *self._zero_page();
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
                self.pc += 1;
            }
            opcodes::LDX_B6 => { // LDX $nn,Y
                self.x = *self._zero_page_y();
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
                self.pc += 1;
            }

            //
            // LOAD - LDY
            //
            opcodes::LDY_A0 => { // LDY #$nn
                self.y = self._immediate();
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
                self.pc += 1;
            }
            opcodes::LDY_AC => { // LDY $nnnn
                self.y = *self._absolute();
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
                self.pc += 2;
            }
            opcodes::LDY_BC => { // LDY $nnnn,X
                self.y = *self._absolute_x();
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
                self.pc += 2;
            }
            opcodes::LDY_A4 => { // LDY $nn
                self.y = *self._zero_page();
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
                self.pc += 1;
            }
            opcodes::LDY_B4 => { // LDY $nn,X
                self.y = *self._zero_page_x();
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
                self.pc += 1;
            }

            //
            // LOAD - STA
            //
            opcodes::STA_8D => { // STA $nnnn
                *self._absolute() = self.a;
                self.pc += 2;
            }
            opcodes::STA_9D => { // STA $nnnn,X
                *self._absolute_x() = self.a;
                self.pc += 2;
            }
            opcodes::STA_99 => { // STA $nnnn,Y
                *self._absolute_y() = self.a;
                self.pc += 2;
            }
            opcodes::STA_85 => { // STA $nn
                *self._zero_page() = self.a;
                self.pc += 1;
            }
            opcodes::STA_95 => { // STA $nn,X
                *self._zero_page_x() = self.a;
                self.pc += 1;
            }
            opcodes::STA_81 => { // STA ($nn,X)
                *self._zero_page_x_indirect() = self.a;
                self.pc += 1;
            }
            opcodes::STA_91 => { // STA ($nn),Y
                *self._zero_page_indirect_y() = self.a;
                self.pc += 1;
            }

            //
            // LOAD - STX
            //
            opcodes::STX_8E => { // STX $nnnn
                *self._absolute() = self.x;
                self.pc += 2;
            }
            opcodes::STX_86 => { // STX $nn
                *self._zero_page() = self.x;
                self.pc += 1;
            }
            opcodes::STX_96 => { // STX $nn,Y
                *self._zero_page_y() = self.x;
                self.pc += 1;
            }

            //
            // LOAD - STY
            //
            opcodes::STY_8C => { // STY $nnnn
                *self._absolute() = self.y;
                self.pc += 2;
            }
            opcodes::STY_84 => { // STY $nn
                *self._zero_page() = self.y;
                self.pc += 1;
            }
            opcodes::STY_94 => { // STY $nn,X
                *self._zero_page_x() = self.y;
                self.pc += 1;
            }

            //
            // NOP
            //
            opcodes::NOP_EA => {
                // NOP, 2 cycles
            }

            //
            // INCREMENT - INC
            //
            opcodes::INC_EE => { // INC $nnnn
                let mut a = *self._absolute();
                a = a.wrapping_add(1);
                self.update_negative(a & 0x80 != 0);
                self.update_zero(a == 0);
                *self._absolute() = a;
                self.pc += 2;
            }
            opcodes::INC_FE => { // INC $nnnn,X
                let mut a = *self._absolute_x();
                a = a.wrapping_add(1);
                self.update_negative(a & 0x80 != 0);
                self.update_zero(a == 0);
                *self._absolute_x() = a;
                self.pc += 2;
            }
            opcodes::INC_E6 => { // INC $nn
                let mut a = *self._zero_page();
                a = a.wrapping_add(1);
                self.update_negative(a & 0x80 != 0);
                self.update_zero(a == 0);
                *self._zero_page() = a;
                self.pc += 1;
            }
            opcodes::INC_F6 => { // INC $nn,X
                let mut a = *self._zero_page_x();
                a = a.wrapping_add(1);
                self.update_negative(a & 0x80 != 0);
                self.update_zero(a == 0);
                *self._zero_page_x() = a;
                self.pc += 1;
            }

            //
            // INCREMENT - INX
            //
            opcodes::INX_E8 => {
                // INX, 2 cycles, Flags: n,z
                let result: u16 = self.x as u16 + 1;

                if result & 0xff == 0 {
                    self.p |= Flags::Z_Zero;
                } else {
                    self.p &= !Flags::Z_Zero;
                }

                self.p &= !Flags::N_Negative;
                self.p |= (result & Flags::N_Negative as u16) as u8;

                self.x = result as u8;
            }

            //
            // INCREMENT - INY
            //
            opcodes::INY_C8 => {
                // INY, 2cycles, Flags: n,z
                let result: u16 = self.y as u16 + 1;

                if result & 0xff == 0 {
                    self.p |= Flags::Z_Zero;
                } else {
                    self.p &= !Flags::Z_Zero;
                }

                self.p &= !Flags::N_Negative;
                self.p |= (result & Flags::N_Negative as u16) as u8;

                self.y = result as u8;
            }

            //
            // ARITH - ADC
            //
            opcodes::ADC_69 => { // ADC #$nn
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = self._immediate();
                let r: u16 = self.a as u16 + v as u16 + c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::ADC_6D => { // ADC $nnnn
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._absolute();
                let r: u16 = self.a as u16 + v as u16 + c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 2;
            }
            opcodes::ADC_7D => { // ADC $nnnn,X
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._absolute_x();
                let r = self.a as u16 + v as u16 + c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 2;
            }
            opcodes::ADC_79 => { // ADC $nnnn,Y
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._absolute_y();
                let r: u16 = self.a as u16 + v as u16 + c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 2;
            }
            opcodes::ADC_65 => { // ADC $nn
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._zero_page();
                let r: u16 = self.a as u16 + v as u16 + c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::ADC_75 => { // ADC $nn,X
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._zero_page_x();
                let r: u16 = self.a as u16 + v as u16 + c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::ADC_61 => { // ADC ($nn,X)
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._zero_page_x_indirect();
                let r: u16 = self.a as u16 + v as u16 + c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::ADC_71 => { // ADC ($nn),Y
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._zero_page_indirect_y();
                let r: u16 = self.a as u16 + v as u16 + c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }

            //
            // ARITH - SBC
            //
            opcodes::SBC_E9 => { // SBC #$nn
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = self._immediate();
                let r: u16 = 0x0100 + self.a as u16 - v as u16 - c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::SBC_ED => { // SBC $nnnn
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._absolute();
                let r: u16 = 0x0100 + self.a as u16 - v as u16 - c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 2;
            }
            opcodes::SBC_FD => { // SBC $nnnn,X
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._absolute_x();
                let r = 0x0100 + self.a as u16 - v as u16 - c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 2;
            }
            opcodes::SBC_F9 => { // SBC $nnnn,Y
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._absolute_y();
                let r: u16 = 0x0100 + self.a as u16 - v as u16 - c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 2;
            }
            opcodes::SBC_E5 => { // SBC $nn
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._zero_page();
                let r: u16 = 0x0100 + self.a as u16 - v as u16 - c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::SBC_F5 => { // SBC $nn,X
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._zero_page_x();
                let r: u16 = 0x0100 + self.a as u16 - v as u16 - c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::SBC_E1 => { // SBC ($nn,X)
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._zero_page_x_indirect();
                let r: u16 = 0x0100 + self.a as u16 - v as u16 - c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::SBC_F1 => { // SBC ($nn),Y
                if self.is_decimal() {
                    unimplemented!();
                }
                let c: u16 = if self.is_carry() {
                    1
                } else {
                    0
                };
                let v = *self._zero_page_indirect_y();
                let r: u16 = 0x0100 + self.a as u16 - v as u16 - c;
                self.a = r as u8;
                // TODO: Add overflow flag update
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }

            //
            // ARITH - CMP
            //
            opcodes::CMP_C9 => { // CMP #$nn
                let v = self._immediate();
                let r: u16 = (0x0100 + self.a as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::CMP_CD => { // CMP $nnnn
                let v = *self._absolute();
                let r: u16 = (0x0100 + self.a as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 2;
            }
            opcodes::CMP_DD => { // CMP $nnnn,X
                let v = *self._absolute_x();
                let r: u16 = (0x0100 + self.a as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 2;
            }
            opcodes::CMP_D9 => { // CMP $nnnn,Y
                let v = *self._absolute_y();
                let r: u16 = (0x0100 + self.a as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 2;
            }
            opcodes::CMP_C5 => { // CMP $nn
                let v = *self._zero_page();
                let r: u16 = (0x0100 + self.a as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::CMP_D5 => { // CMP $nn,X
                let v = *self._zero_page_x();
                let r: u16 = (0x0100 + self.a as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::CMP_C1 => { // CMP ($nn,X)
                let v = *self._zero_page_x_indirect();
                let r: u16 = (0x0100 + self.a as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::CMP_D1 => { // CMP ($nn),Y
                let v = *self._zero_page_indirect_y();
                let r: u16 = (0x0100 + self.a as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }

            //
            // ARITH - CPX
            //
            // FIXME: 0x30 - 0x40 ? -0x10 ? should the N_Negative flags be set or not?
            opcodes::CPX_E0 => { // CPX #$nn
                let v = self._immediate();
                let r: u16 = (0x0100 + self.x as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::CPX_EC => { // CPX $nnnn
                let v = *self._absolute();
                let r: u16 = (0x0100 + self.x as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 2;
            }
            opcodes::CPX_E4 => { // CPX $nn
                let v = *self._zero_page();
                let r: u16 = (0x0100 + self.x as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }

            //
            // ARITH - CPY
            //
            opcodes::CPY_C0 => { // CPY #$nn
                let v = self._immediate();
                let r: u16 = (0x0100 + self.y as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }
            opcodes::CPY_CC => { // CPY $nnnn
                let v = *self._absolute();
                let r: u16 = (0x0100 + self.y as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 2;
            }
            opcodes::CPY_C4 => { // CPY $nn
                let v = *self._zero_page();
                let r: u16 = (0x0100 + self.y as u16) - v as u16;
                self.update_negative(r & 0x80 != 0);
                self.update_zero(r & 0xff == 0);
                self.update_carry(r & 0x0100 != 0);
                self.pc += 1;
            }

            //
            // INCREMENT - DEC
            //
            opcodes::DEC_CE => { // DEC $nnnn
                let mut a = *self._absolute();
                a = a.wrapping_sub(1);
                self.update_negative(a & 0x80 != 0);
                self.update_zero(a == 0);
                *self._absolute() = a;
                self.pc += 2;
            }
            opcodes::DEC_DE => { // DEC $nnnn,X
                let mut a = *self._absolute_x();
                a = a.wrapping_sub(1);
                self.update_negative(a & 0x80 != 0);
                self.update_zero(a == 0);
                *self._absolute_x() = a;
                self.pc += 2;
            }
            opcodes::DEC_C6 => { // DEC $nn
                let mut a = *self._zero_page();
                a = a.wrapping_sub(1);
                self.update_negative(a & 0x80 != 0);
                self.update_zero(a == 0);
                *self._zero_page() = a;
                self.pc += 1;
            }
            opcodes::DEC_D6 => { // DEC $nn,X
                let mut a = *self._zero_page_x();
                a = a.wrapping_sub(1);
                self.update_negative(a & 0x80 != 0);
                self.update_zero(a == 0);
                *self._zero_page_x() = a;
                self.pc += 1;
            }

            //
            // INCREMENT - DEX
            //
            opcodes::DEX_CA => {
                // DEX, 2 cycles, Flags: n,z
                let result: u16 = 0x0100 + self.x as u16 - 1;

                if result as u8 == 0 {
                    self.p |= Flags::Z_Zero;
                } else {
                    self.p &= !Flags::Z_Zero;
                }

                self.p &= !Flags::N_Negative;
                self.p |= (result & Flags::N_Negative as u16) as u8;

                self.x = result as u8;
            }

            //
            // INCREMENT - DEY
            //
            opcodes::DEY_88 => {
                // DEY, 2 cycles, Flags: n,z
                let result: u16 = 0x0100 + self.y as u16 - 1;

                if result as u8 == 0 {
                    self.p |= Flags::Z_Zero;
                } else {
                    self.p &= !Flags::Z_Zero;
                }

                self.p &= !Flags::N_Negative;
                self.p |= (result & Flags::N_Negative as u16) as u8;

                self.y = result as u8;
            }

            //
            // FLAGS - CLC
            //
            opcodes::CLC_18 => {
                self.p &= !Flags::C_Carry;
            }

            //
            // FLAGS - CLD
            //
            opcodes::CLD_D8 => {
                self.p &= !Flags::D_Decimal;
            }

            //
            // FLAGS - CLI
            //
            opcodes::CLI_58 => {
                self.p &= !Flags::I_InterruptDisable;
            }

            //
            // FLAGS - CLV
            //
            opcodes::CLV_B8 => {
                self.p &= !Flags::V_Overflow;
            }

            //
            // FLAGS - SEC
            //
            opcodes::SEC_38 => {
                self.p |= Flags::C_Carry;
            }

            //
            // FLAGS - SED
            //
            opcodes::SED_F8 => {
                self.p |= Flags::D_Decimal;
            }

            //
            // FLAGS - SEI
            //
            opcodes::SEI_78 => {
                self.p |= Flags::I_InterruptDisable;
            }

            //
            // TRANSFER - TAX
            //
            opcodes::TAX_AA => {
                self.x = self.a;
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
            }

            //
            // TRANSFER - TAY
            //
            opcodes::TAY_A8 => {
                self.y = self.a;
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
            }

            //
            // TRANSFER - TSX
            //
            opcodes::TSX_BA => {
                self.x = self.s;
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
            }

            //
            // TRANSFER - TXA
            //
            opcodes::TXA_8A => {
                self.a = self.x;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }

            //
            // TRANSFER - TXS
            //
            opcodes::TXS_9A => {
                self.s = self.x;
                self.update_negative(self.s & 0x80 != 0);
                self.update_zero(self.s == 0);
            }

            //
            // TRANSFER - TYA
            //
            opcodes::TYA_98 => {
                self.a = self.y;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }

            //
            // STACK - PHA
            //
            opcodes::PHA_48 => {
                self.memory[0x100 + self.s as usize] = self.a;
                self.s -= 1;
            }

            //
            // STACK - PHP
            //
            opcodes::PHP_08 => {
                self.memory[0x100 + self.s as usize] = self.p;
                self.s -= 1;
            }

            //
            // STACK - PLA
            //
            opcodes::PLA_68 => {
                self.s += 1;
                self.a = self.memory[0x100 + self.s as usize];
            }

            //
            // STACK - PLP
            //
            opcodes::PLP_28 => {
                self.s += 1;
                self.p = self.memory[0x100 + self.s as usize];
            }

            //
            // CONTROL
            //
            opcodes::BRK_00 => {
                unimplemented!();
            }
            opcodes::JMP_4C => { // JMP $nnnn
                let addr = self._absolute_addr();
                self.pc = addr as u16;
            }
            opcodes::JMP_6C => { // JMP ($nnnn)
                let addr = self._absolute_indirect_addr();
                self.pc = addr as u16;
            }
            opcodes::JSR_20 => { // JSR $nnnn
                // store program counter on stack
                let pc = self.pc + 1; // +1 -> the last byte of this 3byte instruction. TODO: pay attention
                self.memory[0x0100 + self.s as usize] = (pc >> 8) as u8;
                self.s -= 1;
                self.memory[0x0100 + self.s as usize] = pc as u8;
                self.s -= 1;

                // load new program counter
                self.pc = self._absolute_addr() as u16;
            }
            opcodes::RTI_40 => {
                // restore status flags
                self.p = self.memory[0x0100 + self.s as usize];
                self.s += 1;

                // restore pc: low byte
                self.pc = self.memory[0x0100 + self.s as usize] as u16;
                self.s += 1;
                // restore pc: high byte
                self.pc |= (self.memory[0x0100 + self.s as usize] as u16) << 8;
                self.s += 1;

                self.pc += 1;
            }
            opcodes::RTS_60 => {
                // low byte
                self.s += 1;
                self.pc = self.memory[0x0100 + self.s as usize] as u16;
                // high byte
                self.s += 1;
                self.pc |= (self.memory[0x0100 + self.s as usize] as u16) << 8;

                self.pc += 1;
            }


            //
            // BRANCH - BCC - Branch Carry Clear
            //
            opcodes::BCC_90 => {
                if !self.is_carry() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }

            //
            // BRANCH - BCS - Branch Carry Set
            //
            opcodes::BCS_B0 => {
                if self.is_carry() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }

            //
            // BRANCH - BEQ - Branch Equal
            //
            opcodes::BEQ_F0 => {
                if self.is_zero() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }

            //
            // BRANCH - BMI - Branch MInus
            //
            opcodes::BMI_30 => {
                if self.is_negative() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }

            //
            // BRANCH - BNE - Branch Not Equal
            //
            opcodes::BNE_D0 => {
                if !self.is_zero() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }

            //
            // BRANCH - BPL - Branch PLus
            //
            opcodes::BPL_10 => {
                if !self.is_negative() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }

            //
            // BRANCH - BVC - Branch oVerflow Clear
            //
            opcodes::BVC_50 => {
                if !self.is_overflow() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }

            //
            // BRANCH - BVS - Branch oVerflow Set
            //
            opcodes::BVS_70 => {
                if self.is_overflow() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }

            //
            // SHIFT - ASL - Arithmetic Shift Left
            //
            // TODO: Use `_addressing_mode()` functions here
            opcodes::ASL_0A => { // ASL A
                self.a = self._asl_inst(self.a);
            }
            opcodes::ASL_0E => { // ASL $nnnn
                let addr = self.get_addr();
                self.memory[addr] = self._asl_inst(self.memory[addr]);
                //let mut addr = self._absolute();
                //*addr = self._asl_inst(*addr);
                self.pc += 2;
            }
            opcodes::ASL_1E => { // ASL $nnnn,X
                let addr = self.get_addr() + self.x as usize;
                self.memory[addr] = self._asl_inst(self.memory[addr]);
                //self._asl_inst(self._absolute_x());
                self.pc += 2;
            }
            opcodes::ASL_06 => { // ASL $nn
                let addr = self.get_addr_zero_page();
                self.memory[addr] = self._asl_inst(self.memory[addr]);
                //self._asl_inst(self._zero_page());
                self.pc += 1;
            }
            opcodes::ASL_16 => { // ASL $nn,X
                let addr = (self.get_addr_zero_page() + self.x as usize) & 0xff;
                self.memory[addr] = self._asl_inst(self.memory[addr]);
                //self._asl_inst(self._zero_page_x());
                self.pc += 1;
            }

            //
            // SHIFT - LSR - Logic Shift Right
            //
            // TODO: Use `_addressing_mode()` functions here
            opcodes::LSR_4A => {
                // LSR A
                self.a = self._lsr_inst(self.a);
            }
            opcodes::LSR_4E => {
                // LSR $nnnn
                let addr = self.get_addr();
                self.memory[addr] = self._lsr_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::LSR_5E => {
                // LSR $nnnn,X
                let mut addr = self.get_addr();
                addr += self.x as usize;
                self.memory[addr] = self._lsr_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::LSR_46 => {
                // LSR $nn
                let addr = self.get_addr_zero_page();
                self.memory[addr] = self._lsr_inst(self.memory[addr]);
                self.pc += 1;
            }
            opcodes::LSR_56 => {
                // LSR $nn,X
                let addr = (self.get_addr_zero_page() + self.x as usize) & 0xff;
                self.memory[addr] = self._lsr_inst(self.memory[addr]);
                self.pc += 1;
            }

            //
            // SHIFT - ROL - Rotate Left
            //
            // TODO: Use `_addressing_mode()` functions here
            opcodes::ROL_2A => {
                // ROL A
                self.a = self._rol_inst(self.a);
            }
            opcodes::ROL_2E => {
                // ROL $nnnn
                let addr = self.get_addr();
                self.memory[addr] = self._rol_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::ROL_3E => {
                // ROL $nnnn,X
                let addr = self.get_addr() + self.x as usize;
                self.memory[addr] = self._rol_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::ROL_26 => {
                // ROL $nn
                let addr = self.get_addr_zero_page();
                self.memory[addr] = self._rol_inst(self.memory[addr]);
                self.pc += 1;
            }
            opcodes::ROL_36 => {
                // ROL $nn,X
                let addr = (self.get_addr_zero_page() + self.x as usize) & 0xff;
                self.memory[addr] = self._rol_inst(self.memory[addr]);
                self.pc += 1;
            }

            //
            // SHIFT - ROR - Rotate Right
            //
            // TODO: Use `_addressing_mode()` functions here
            opcodes::ROR_6A => {
                // ROR A
                self.a = self._ror_inst(self.a);
            }
            opcodes::ROR_6E => {
                // ROR $nnnn
                let addr = self.get_addr();
                self.memory[addr] = self._ror_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::ROR_7E => {
                // ROR $nnnn,X
                let addr = self.get_addr() + self.x as usize;
                self.memory[addr] = self._ror_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::ROR_66 => {
                // ROR $nn
                let addr = self.get_addr_zero_page();
                self.memory[addr] = self._ror_inst(self.memory[addr]);
                self.pc += 1;
            }
            opcodes::ROR_76 => {
                // ROR $nn,X
                let addr = (self.get_addr_zero_page() + self.x as usize) & 0xff;
                self.memory[addr] = self._ror_inst(self.memory[addr]);
                self.pc += 1;
            }

            //
            // LOGIC - AND
            //
            // TODO: Try to remove the usage of `v` here (and copy semantics)
            opcodes::AND_29 => { // AND #$nn
                let v = self._immediate();
                self._and_inst(v);
                self.pc += 1;
            }
            opcodes::AND_2D => { // AND $nnnn
                let v = *self._absolute();
                self._and_inst(v);
                self.pc += 2;
            }
            opcodes::AND_3D => { // AND $nnnn,X
                let v = *self._absolute_x();
                self._and_inst(v);
                self.pc += 2;
            }
            opcodes::AND_39 => { // AND $nnnn,Y
                let v = *self._absolute_y();
                self._and_inst(v);
                self.pc += 2;
            }
            opcodes::AND_25 => { // AND $nn
                let v = *self._zero_page();
                self._and_inst(v);
                self.pc += 1;
            }
            opcodes::AND_35 => { // AND $nn,X
                let v = *self._zero_page_x();
                self._and_inst(v);
                self.pc += 1;
            }
            opcodes::AND_21 => { // AND ($nn,X)
                let v = *self._zero_page_x_indirect();
                self._and_inst(v);
                self.pc += 1;
            }
            opcodes::AND_31 => { // AND ($nn),Y
                let v = *self._zero_page_indirect_y();
                self._and_inst(v);
                self.pc += 1;
            }

            //
            // BIT
            //
            // TODO: Try to remove the usage of `v` here (and copy semantics)
            opcodes::BIT_2C => { // BIT $nnnn
                let v = *self._absolute();
                self._bit_inst(v);
                self.pc += 2;
            }
            opcodes::BIT_24 => { // BIT $nn
                let v = *self._zero_page();
                self._bit_inst(v);
                self.pc += 1;
            }

            //
            // EOR
            //
            // TODO: Try to remove the usage of `v` here (and copy semantics)
            opcodes::EOR_49 => { // EOR #$49
                let v = self._immediate();
                self._eor_inst(v);
                self.pc += 1;
            }
            opcodes::EOR_4D => { // EOR $nnnn
                let v = *self._absolute();
                self._eor_inst(v);
                self.pc += 2;
            }
            opcodes::EOR_5D => { // EOR $nnnn,X
                let v = *self._absolute_x();
                self._eor_inst(v);
                self.pc += 2;
            }
            opcodes::EOR_59 => { // EOR $nnnn,Y
                let v = *self._absolute_y();
                self._eor_inst(v);
                self.pc += 2;
            }
            opcodes::EOR_45 => { // EOR $nn
                let v = *self._zero_page();
                self._eor_inst(v);
                self.pc += 1;
            }
            opcodes::EOR_55 => { // EOR $nn,X
                let v = *self._zero_page_x();
                self._eor_inst(v);
                self.pc += 1;
            }
            opcodes::EOR_41 => { // EOR ($nn,X)
                let v = *self._zero_page_x_indirect();
                self._eor_inst(v);
                self.pc += 1;
            }
            opcodes::EOR_51 => { // EOR ($nn),Y
                let v = *self._zero_page_indirect_y();
                self._eor_inst(v);
                self.pc += 1;
            }

            //
            // ORA
            //
            // TODO: Try to remove the usage of `v` here (and copy semantics)
            opcodes::ORA_09 => { // ORA #$nn
                let v = self._immediate();
                self._ora_inst(v);
                self.pc += 1;
            }
            opcodes::ORA_0D => { // ORA $nnnn
                let v = *self._absolute();
                self._ora_inst(v);
                self.pc += 2;
            }
            opcodes::ORA_1D => { // ORA $nnnn,X
                let v = *self._absolute_x();
                self._ora_inst(v);
                self.pc += 2;
            }
            opcodes::ORA_19 => { // ORA $nnnn,Y
                let v = *self._absolute_y();
                self._ora_inst(v);
                self.pc += 2;
            }
            opcodes::ORA_05 => { // ORA $nn
                let v = *self._zero_page();
                self._ora_inst(v);
                self.pc += 1;
            }
            opcodes::ORA_15 => { // ORA $nn,X
                let v = *self._zero_page_x();
                self._ora_inst(v);
                self.pc += 1;
            }
            opcodes::ORA_01 => { // ORA ($nn,X)
                let v = *self._zero_page_x_indirect();
                self._ora_inst(v);
                self.pc += 1;
            }
            opcodes::ORA_11 => { // ORA ($nn),Y
                let v = *self._zero_page_indirect_y();
                self._ora_inst(v);
                self.pc += 1;
            }

            _ => unimplemented!(),
        }
    }

    fn run(&mut self) {
        loop {
            self.step();
        }
    }

    // FIXME: Remove it
    fn run_opcode(&mut self, op: &[u8]) {
        let opcode = op[0];
        match opcode {
            opcodes::NOP_EA => {
                // NOP, 2 cycles
            }
            opcodes::INX_E8 => {
                // INX, 2 cycles, Flags: n,z
                let result: u16 = self.x as u16 + 1;

                if result & 0xff == 0 {
                    self.p |= Flags::Z_Zero;
                } else {
                    self.p &= !Flags::Z_Zero;
                }

                self.p &= !Flags::N_Negative;
                self.p |= (result & Flags::N_Negative as u16) as u8;

                self.x = result as u8;
            }
            opcodes::INY_C8 => {
                // INY, 2cycles, Flags: n,z
                let result: u16 = self.y as u16 + 1;

                if result & 0xff == 0 {
                    self.p |= Flags::Z_Zero;
                } else {
                    self.p &= !Flags::Z_Zero;
                }

                self.p &= !Flags::N_Negative;
                self.p |= (result & Flags::N_Negative as u16) as u8;

                self.y = result as u8;
            }
            opcodes::DEX_CA => {
                // DEX, 2 cycles, Flags: n,z
                let result: u16 = 0x0100 + self.x as u16 - 1;

                if result as u8 == 0 {
                    self.p |= Flags::Z_Zero;
                } else {
                    self.p &= !Flags::Z_Zero;
                }

                self.p &= !Flags::N_Negative;
                self.p |= (result & Flags::N_Negative as u16) as u8;

                self.x = result as u8;
            }
            opcodes::DEY_88 => {
                // DEY, 2 cycles, Flags: n,z
                let result: u16 = 0x0100 + self.y as u16 - 1;

                if result as u8 == 0 {
                    self.p |= Flags::Z_Zero;
                } else {
                    self.p &= !Flags::Z_Zero;
                }

                self.p &= !Flags::N_Negative;
                self.p |= (result & Flags::N_Negative as u16) as u8;

                self.y = result as u8;
            }

            // flags
            opcodes::CLC_18 => {
                self.p &= !Flags::C_Carry;
            }
            opcodes::CLD_D8 => {
                self.p &= !Flags::D_Decimal;
            }
            opcodes::CLI_58 => {
                self.p &= !Flags::I_InterruptDisable;
            }
            opcodes::CLV_B8 => {
                self.p &= !Flags::V_Overflow;
            }

            opcodes::SEC_38 => {
                self.p |= Flags::C_Carry;
            }
            opcodes::SED_F8 => {
                self.p |= Flags::D_Decimal;
            }
            opcodes::SEI_78 => {
                self.p |= Flags::I_InterruptDisable;
            }

            // TRANSFER
            opcodes::TAX_AA => {
                self.x = self.a;
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
            }
            opcodes::TAY_A8 => {
                self.y = self.a;
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
            }
            opcodes::TSX_BA => {
                self.x = self.s;
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
            }
            opcodes::TXA_8A => {
                self.a = self.x;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }
            opcodes::TXS_9A => {
                self.s = self.x;
                self.update_negative(self.s & 0x80 != 0);
                self.update_zero(self.s == 0);
            }
            opcodes::TYA_98 => {
                self.a = self.y;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }

            // STACK
            opcodes::PHA_48 => {
                self.memory[0x100 + self.s as usize] = self.a;
                self.s -= 1;
            }
            opcodes::PHP_08 => {
                self.memory[0x100 + self.s as usize] = self.p;
                self.s -= 1;
            }
            opcodes::PLA_68 => {
                self.s += 1;
                self.a = self.memory[0x100 + self.s as usize];
            }
            opcodes::PLP_28 => {
                self.s += 1;
                self.p = self.memory[0x100 + self.s as usize];
            }

            _ => unimplemented!(),
        }
    }

    fn patch_memory(&mut self, offset: usize, bytes: &[u8]) {
        for (idx, b) in bytes.iter().enumerate() {
            self.memory[offset + idx] = *b;
        }
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
        cpu.run_opcode(&[NOP_EA]);

        let fresh = Cpu::new();
        assert_eq!(cpu, fresh);
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
        cpu.x = 0x35;
        let mut expected: u8 = 200u8;

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = INC_FE; // INC $4000
        mem[1] = 0x20;
        mem[2] = 0x40;
        mem[0x4055] = 200;
        cpu.patch_memory(0, &mem);

        for _ in 1..512 {
            cpu.pc = 0x0000;
            cpu.step();
            expected = expected.wrapping_add(1);

            assert_eq!(cpu.memory[0x4055], expected);

            let expected_z: bool = expected == 0;
            let expected_n = (expected & 0x80) > 0;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative > 0, expected_n);
        }
    }

    #[test]
    fn test_inc_e6() { // INC $nn
        let mut cpu = Cpu::new();
        let mut expected: u8 = 200u8;

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = INC_E6; // INC $4000
        mem[1] = 0x20;
        mem[0x0020] = 200;
        cpu.patch_memory(0, &mem);

        for _ in 1..512 {
            cpu.pc = 0x0000;
            cpu.step();
            expected = expected.wrapping_add(1);

            assert_eq!(cpu.memory[0x0020], expected);

            let expected_z: bool = expected == 0;
            let expected_n = (expected & 0x80) > 0;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative > 0, expected_n);
        }
    }

    #[test]
    fn test_inc_f6() { // INC $nn,X
        let mut cpu = Cpu::new();
        cpu.x = 0x35;
        let mut expected: u8 = 200u8;

        let mut mem: [u8; MEM_SZ] = [0; MEM_SZ];
        mem[0] = INC_F6; // INC $4000
        mem[1] = 0x20;
        mem[0x0055] = 200;
        cpu.patch_memory(0, &mem);

        for _ in 1..512 {
            cpu.pc = 0x0000;
            cpu.step();
            expected = expected.wrapping_add(1);

            assert_eq!(cpu.memory[0x0055], expected);

            let expected_z: bool = expected == 0;
            let expected_n = (expected & 0x80) > 0;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative > 0, expected_n);
        }
    }

    //
    // INX
    //
    #[test]
    fn test_inx() {
        let mut cpu = Cpu::new();
        let mut expected_x: u16 = 0;

        assert_eq!(cpu.x, expected_x as u8);

        cpu.run_opcode(&[INX_E8]);
        expected_x += 1;

        assert_eq!(cpu.x, expected_x as u8);
        assert_eq!(cpu.p, 0);

        for _ in 1..512 {
            cpu.run_opcode(&[INX_E8]);
            expected_x += 1;

            assert_eq!(cpu.x, expected_x as u8);

            let expected_z: bool = if (expected_x as u8) == 0 { true } else { false };
            let expected_n = expected_x as u8 & 0x80;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative, expected_n);
        }
    }

    #[test]
    fn test_iny() {
        // this is a copy of inx test
        let mut cpu = Cpu::new();
        let mut expected_y: u16 = 0;

        assert_eq!(cpu.y, expected_y as u8);

        cpu.run_opcode(&[INY_C8]);
        expected_y += 1;

        assert_eq!(cpu.y, expected_y as u8);
        assert_eq!(cpu.p, 0);

        for _ in 1..512 {
            cpu.run_opcode(&[INY_C8]);
            expected_y += 1;

            assert_eq!(cpu.y, expected_y as u8);

            let expected_z: bool = if (expected_y as u8) == 0 { true } else { false };
            let expected_n = expected_y as u8 & 0x80;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative, expected_n);
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
            println!("addr:0x{addr:04x}  addr2:0x{addr2:04x}  y:{y:02x}  addr3:0x{addr3:04x}  a:{a}  v:{v}  carry:{carry}  exp_a:{exp_a}  exp_flags: {exp_flags:02x}");
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

            println!("cpu.a: {}", cpu.a);
            println!("exp_a: {}", exp_a);
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
        cpu.x = 100;
        let mut expected_x: i16 = 100;

        assert_eq!(cpu.x, expected_x as u8);

        cpu.run_opcode(&[0xCA]);
        expected_x -= 1;

        assert_eq!(cpu.x, expected_x as u8);
        assert_eq!(cpu.p, 0);

        for _ in 1..512 {
            cpu.run_opcode(&[DEX_CA]);
            expected_x -= 1;

            assert_eq!(cpu.x, expected_x as u8);

            let expected_z: bool = if (expected_x as u8) == 0 { true } else { false };
            let expected_n = expected_x as u8 & 0x80;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative, expected_n);
        }
    }

    #[test]
    fn test_dey() {
        // copy-pasta of test_dex()
        let mut cpu = Cpu::new();
        cpu.y = 100;
        let mut expected_y: i16 = 100;

        assert_eq!(cpu.y, expected_y as u8);

        cpu.run_opcode(&[0x88]);
        expected_y -= 1;

        assert_eq!(cpu.y, expected_y as u8);
        assert_eq!(cpu.p, 0);

        for _ in 1..512 {
            cpu.run_opcode(&[opcodes::DEY_88]);
            expected_y -= 1;

            assert_eq!(cpu.y, expected_y as u8);

            let expected_z: bool = if (expected_y as u8) == 0 { true } else { false };
            let expected_n = expected_y as u8 & 0x80;
            assert_eq!((cpu.p & Flags::Z_Zero) > 0, expected_z);
            assert_eq!(cpu.p & Flags::N_Negative, expected_n);
        }
    }

    //
    // FLAGS
    //
    #[test]
    fn test_clc() {
        // clear carry flag
        let mut cpu = Cpu::new();
        cpu.p = Flags::C_Carry;
        assert!(cpu.p & Flags::C_Carry != 0);

        cpu.run_opcode(&[CLC_18]);
        assert!(cpu.p & Flags::C_Carry == 0);
    }

    #[test]
    fn test_cld() {
        let mut cpu = Cpu::new();
        cpu.p |= Flags::D_Decimal;
        assert!(cpu.p & Flags::D_Decimal != 0);

        cpu.run_opcode(&[CLD_D8]);
        assert!(cpu.p & Flags::D_Decimal == 0);
    }

    #[test]
    fn test_cli() {
        let mut cpu = Cpu::new();
        cpu.p |= Flags::I_InterruptDisable;
        assert!(cpu.p & Flags::I_InterruptDisable != 0);

        cpu.run_opcode(&[CLI_58]);
        assert!(cpu.p & Flags::I_InterruptDisable == 0);
    }

    #[test]
    fn test_clv() {
        let mut cpu = Cpu::new();
        cpu.p |= Flags::V_Overflow;
        assert!(cpu.p & Flags::V_Overflow != 0);

        cpu.run_opcode(&[CLV_B8]);
        assert!(cpu.p & Flags::V_Overflow == 0);
    }

    #[test]
    fn test_sec() {
        let mut cpu = Cpu::new();
        assert!(cpu.p & Flags::C_Carry == 0);

        cpu.run_opcode(&[SEC_38]);
        assert!(cpu.p & Flags::C_Carry != 0);
    }

    #[test]
    fn test_sed() {
        let mut cpu = Cpu::new();
        assert!(cpu.p & Flags::D_Decimal == 0);

        cpu.run_opcode(&[SED_F8]);
        assert!(cpu.p & Flags::D_Decimal != 0);
    }

    #[test]
    fn test_sei() {
        let mut cpu = Cpu::new();
        assert!(cpu.p & Flags::I_InterruptDisable == 0);

        cpu.run_opcode(&[SEI_78]);
        assert!(cpu.p & Flags::I_InterruptDisable != 0);
    }

    //
    // TRANSFER
    //
    #[test]
    fn test_tax() {
        let mut cpu = Cpu::new();
        cpu.a = 55;

        assert!(cpu.a == 55);
        assert!(cpu.x == 0);

        cpu.run_opcode(&[TAX_AA]);
        assert!(cpu.a == 55);
        assert!(cpu.x == 55);
        assert!(!cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.a = 200;
        assert!(cpu.a == 200);
        assert!(cpu.x == 55);

        cpu.run_opcode(&[TAX_AA]);
        assert!(cpu.a == 200);
        assert!(cpu.x == 200);
        assert!(cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.a = 0;
        assert!(cpu.a == 0);
        assert!(cpu.x == 200);

        cpu.run_opcode(&[TAX_AA]);
        assert!(cpu.a == 0);
        assert!(cpu.x == 0);
        assert!(!cpu.is_negative());
        assert!(cpu.is_zero());
    }

    #[test]
    fn test_tay() {
        let mut cpu = Cpu::new();
        cpu.a = 55;

        assert!(cpu.a == 55);
        assert!(cpu.y == 0);

        cpu.run_opcode(&[TAY_A8]);
        assert!(cpu.a == 55);
        assert!(cpu.y == 55);
        assert!(!cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.a = 200;
        assert!(cpu.a == 200);
        assert!(cpu.y == 55);

        cpu.run_opcode(&[TAY_A8]);
        assert!(cpu.a == 200);
        assert!(cpu.y == 200);
        assert!(cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.a = 0;
        assert!(cpu.a == 0);
        assert!(cpu.y == 200);

        cpu.run_opcode(&[TAY_A8]);
        assert!(cpu.a == 0);
        assert!(cpu.y == 0);
        assert!(!cpu.is_negative());
        assert!(cpu.is_zero());
    }

    #[test]
    fn test_tsx() {
        let mut cpu = Cpu::new();
        cpu.s = 55;

        assert!(cpu.s == 55);
        assert!(cpu.x == 0);

        cpu.run_opcode(&[TSX_BA]);
        assert!(cpu.s == 55);
        assert!(cpu.x == 55);
        assert!(!cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.s = 200;
        assert!(cpu.s == 200);
        assert!(cpu.x == 55);

        cpu.run_opcode(&[TSX_BA]);
        assert!(cpu.s == 200);
        assert!(cpu.x == 200);
        assert!(cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.s = 0;
        assert!(cpu.s == 0);
        assert!(cpu.x == 200);

        cpu.run_opcode(&[TSX_BA]);
        assert!(cpu.s == 0);
        assert!(cpu.x == 0);
        assert!(!cpu.is_negative());
        assert!(cpu.is_zero());
    }

    #[test]
    fn test_txa() {
        let mut cpu = Cpu::new();
        cpu.x = 55;

        assert!(cpu.x == 55);
        assert!(cpu.a == 0);

        cpu.run_opcode(&[TXA_8A]);
        assert!(cpu.x == 55);
        assert!(cpu.a == 55);
        assert!(!cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.x = 200;
        assert!(cpu.x == 200);
        assert!(cpu.a == 55);

        cpu.run_opcode(&[TXA_8A]);
        assert!(cpu.x == 200);
        assert!(cpu.a == 200);
        assert!(cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.x = 0;
        assert!(cpu.x == 0);
        assert!(cpu.a == 200);

        cpu.run_opcode(&[TXA_8A]);
        assert!(cpu.x == 0);
        assert!(cpu.a == 0);
        assert!(!cpu.is_negative());
        assert!(cpu.is_zero());
    }

    #[test]
    fn test_txs() {
        let mut cpu = Cpu::new();
        cpu.x = 55;

        assert!(cpu.x == 55);
        assert!(cpu.s == 0xff);

        cpu.run_opcode(&[TXS_9A]);
        assert!(cpu.x == 55);
        assert!(cpu.s == 55);
        assert!(!cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.x = 200;
        assert!(cpu.x == 200);
        assert!(cpu.s == 55);

        cpu.run_opcode(&[TXS_9A]);
        assert!(cpu.x == 200);
        assert!(cpu.s == 200);
        assert!(cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.x = 0;
        assert!(cpu.x == 0);
        assert!(cpu.s == 200);

        cpu.run_opcode(&[TXS_9A]);
        assert!(cpu.x == 0);
        assert!(cpu.s == 0);
        assert!(!cpu.is_negative());
        assert!(cpu.is_zero());
    }

    #[test]
    fn test_tya() {
        let mut cpu = Cpu::new();
        cpu.y = 55;

        assert!(cpu.y == 55);
        assert!(cpu.a == 0);

        cpu.run_opcode(&[TYA_98]);
        assert!(cpu.y == 55);
        assert!(cpu.a == 55);
        assert!(!cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.y = 200;
        assert!(cpu.y == 200);
        assert!(cpu.a == 55);

        cpu.run_opcode(&[TYA_98]);
        assert!(cpu.y == 200);
        assert!(cpu.a == 200);
        assert!(cpu.is_negative());
        assert!(!cpu.is_zero());

        // --------------------------------------

        cpu.y = 0;
        assert!(cpu.y == 0);
        assert!(cpu.a == 200);

        cpu.run_opcode(&[TYA_98]);
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
        cpu.a = 0x28;

        assert!(cpu.s == 0xff);
        cpu.run_opcode(&[PHA_48]);

        assert!(cpu.s == 0xfe);
        assert!(cpu.a == 0x28);
        assert!(cpu.memory[cpu.s as usize + 1 + 0x0100] == 0x28)
    }

    #[test]
    fn test_php() {
        let mut cpu = Cpu::new();

        cpu.run_opcode(&[SEC_38]);
        cpu.run_opcode(&[SED_F8]);
        cpu.run_opcode(&[SEI_78]);

        assert!(cpu.is_carry());
        assert!(cpu.is_decimal());
        assert!(cpu.is_interrupt_disabled());

        cpu.run_opcode(&[PHP_08]);

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

        cpu.run_opcode(&[PHA_48]);
        cpu.a = 0x66;
        assert!(cpu.a == 0x66);
        assert!(cpu.s == 0xfe);

        cpu.run_opcode(&[PLA_68]);
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

        cpu.run_opcode(&[PHA_48]);
        assert!(cpu.s == 0xfe);

        cpu.run_opcode(&[PLP_28]);
        assert!(cpu.s == 0xff);
        assert!(cpu.is_carry());
        assert!(cpu.is_decimal());
        assert!(cpu.is_overflow());
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
