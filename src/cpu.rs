// TODO: Make Cpu reference to memory, not owning it
//       (it may speed up tests a little bit)

// TODO: Make all the tests look the same: all have some _t function
//       All have ready memory, no memory modification during tests,
//          no pre-calculated addresses passed to test

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

mod Flags {
    pub(crate) const N_Negative: u8 = 0x80;
    pub(crate) const V_Overflow: u8 = 0x40;
    // the 0x20 bit is unused
    pub(crate) const B_Break: u8 = 0x10;
    pub(crate) const D_Decimal: u8 = 0x08;
    pub(crate) const I_InterruptDisable: u8 = 0x04;
    pub(crate) const Z_Zero: u8 = 0x02;
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
    pub const AND: u8 = 0x00;
    pub const BIT: u8 = 0x00;
    pub const EOR: u8 = 0x00;
    pub const ORA: u8 = 0x00;

    // arith
    pub const ADC: u8 = 0x00;
    pub const CMP: u8 = 0x00;
    pub const CPX: u8 = 0x00;
    pub const CPY: u8 = 0x00;
    pub const SBC: u8 = 0x00;

    // increment
    pub const DEC: u8 = 0x00; // TODO
    pub const DEX_CA: u8 = 0xCA;
    pub const DEY_88: u8 = 0x88;
    pub const INC: u8 = 0x00; // TODO
    pub const INX_E8: u8 = 0xE8;
    pub const INY_C8: u8 = 0xC8;

    // control
    pub const BRK: u8 = 0x00;
    pub const JMP: u8 = 0x00;
    pub const JSR: u8 = 0x00;
    pub const RTI: u8 = 0x00;
    pub const RTS: u8 = 0x00;

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

    /*
    fn update_negative(&mut self, val: u8) {
        if val & Flags::N_Negative != 0 {
            self.p |= Flags::N_Negative;
        } else {
            self.p &= !Flags::N_Negative;
        }
    }
    */

    fn update_zero(&mut self, flag: bool) {
        if flag {
            self.p |= Flags::Z_Zero;
        } else {
            self.p &= !Flags::Z_Zero;
        }
    }

    /*
    fn update_zero(&mut self, val: u8) {
        if val == 0 {
            self.p |= Flags::Z_Zero;
        } else {
            self.p &= !Flags::Z_Zero;
        }
    }
    */

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
            // LOAD
            //
            // LOAD - LDA
            opcodes::LDA_A9 => {
                self.a = self.memory[self.pc as usize];
                self.pc += 1;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }
            opcodes::LDA_AD => {
                let addr = self.get_addr() as usize;
                self.a = self.memory[addr];

                self.pc += 2;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }
            opcodes::LDA_BD => {
                let addr = self.get_addr() as usize + self.x as usize;
                self.a = self.memory[addr];

                self.pc += 2;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }
            opcodes::LDA_B9 => {
                let addr = self.get_addr() as usize + self.y as usize;
                self.a = self.memory[addr];

                self.pc += 2;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }
            opcodes::LDA_A5 => {
                let addr = self.get_addr_zero_page() as usize ;
                self.a = self.memory[addr];

                self.pc += 1;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }
            opcodes::LDA_B5 => {
                let mut addr = self.get_addr_zero_page() as usize;
                addr = (addr + self.x as usize) & 0xff;
                self.a = self.memory[addr];

                self.pc += 1;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }
            opcodes::LDA_A1 => {
                let mut addr = self.get_addr_zero_page() as usize;
                // println!("nn: {:02x}", addr);
                addr = (addr + self.x as usize) & 0xff;
                // println!("x: {:02x}", self.x);
                // println!("resulting addr1: {:02x}", addr);

                // println!("before cpu.a: {:02x}", self.a);

                let mut addr2 = self.memory[addr] as usize;
                addr2 |= (self.memory[addr + 1] as usize) << 8;
                self.a = self.memory[addr2];

                // let mut addr3 = (self.memory[addr2+1] as usize) << 8;
                // addr3 += self.memory[addr2] as usize;

                // // println!("addr2: {:04x}", addr2);
                // self.a = self.memory[addr3];
                // // println!("after cpu.a: {:02x}", self.a);

                self.pc += 1;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }
            opcodes::LDA_B1 => {
                let mut addr = self.get_addr_zero_page() as usize;

                let mut addr2 = (self.memory[addr + 1] as usize) << 8;
                addr2 += self.memory[addr] as usize + self.y as usize;

                self.a = self.memory[addr2];

                self.pc += 1;
                self.update_negative(self.a & 0x80 != 0);
                self.update_zero(self.a == 0);
            }

            // LOAD - LDX
            opcodes::LDX_A2 => { // LDX #$nn
                self.x = self.memory[self.pc as usize];
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
                self.pc += 1;
            }
            opcodes::LDX_AE => { // LDX $nnnn
                let addr = self.get_addr();
                self.x = self.memory[addr];
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
                self.pc += 2;
            }
            opcodes::LDX_BE => { // LDX $nnnn,Y
                let addr = self.get_addr() + self.y as usize;
                self.x = self.memory[addr];
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
                self.pc += 2;
            }
            opcodes::LDX_A6 => { // LDX $nn
                let addr = self.get_addr_zero_page();
                self.x = self.memory[addr];
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
                self.pc += 1;
            }
            opcodes::LDX_B6 => { // LDX $nn,Y
                let addr = self.get_addr_zero_page() + self.y as usize;
                self.x = self.memory[addr];
                self.update_negative(self.x & 0x80 != 0);
                self.update_zero(self.x == 0);
                self.pc += 1;
            }

            // LOAD - LDY
            opcodes::LDY_A0 => { // LDY #$nn
                self.y = self.memory[self.pc as usize];
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
                self.pc += 1;
            }
            opcodes::LDY_AC => { // LDY $nnnn
                let addr = self.get_addr();
                self.y = self.memory[addr];
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
                self.pc += 2;
            }
            opcodes::LDY_BC => { // LDY $nnnn,X
                let addr = self.get_addr() + self.x as usize;
                self.y = self.memory[addr];
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
                self.pc += 2;
            }
            opcodes::LDY_A4 => { // LDY $nn
                let addr = self.get_addr_zero_page();
                self.y = self.memory[addr];
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
                self.pc += 1;
            }
            opcodes::LDY_B4 => { // LDY $nn,X
                let addr = self.get_addr_zero_page() + self.x as usize;
                self.y = self.memory[addr];
                self.update_negative(self.y & 0x80 != 0);
                self.update_zero(self.y == 0);
                self.pc += 1;
            }

            //
            // NOP
            //
            opcodes::NOP_EA => {
                // NOP, 2 cycles
            }

            //
            // INCREMENT
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

            // BRANCH
            opcodes::BCC_90 => {
                if !self.is_carry() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }
            opcodes::BCS_B0 => {
                if self.is_carry() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }
            opcodes::BEQ_F0 => {
                if self.is_zero() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }
            opcodes::BMI_30 => {
                if self.is_negative() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }
            opcodes::BNE_D0 => {
                if !self.is_zero() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }
            opcodes::BPL_10 => {
                if !self.is_negative() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }
            opcodes::BVC_50 => {
                if !self.is_overflow() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }
            opcodes::BVS_70 => {
                if self.is_overflow() {
                    self.set_pc_to_current_addr_in_memory();
                } else {
                    self.pc += 2;
                }
            }

            // SHIFT ASL
            opcodes::ASL_0A => {
                // ASL A
                self.a = self._asl_inst(self.a);
            }
            opcodes::ASL_0E => {
                // ASL $nnnn
                let addr = self.get_addr();
                self.memory[addr] = self._asl_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::ASL_1E => {
                // ASL $nnnn,X
                let addr = self.get_addr() + self.x as usize;
                self.memory[addr] = self._asl_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::ASL_06 => {
                // ASL $nn
                let addr = self.get_addr_zero_page();
                self.memory[addr] = self._asl_inst(self.memory[addr]);
                self.pc += 1;
            }
            opcodes::ASL_16 => {
                // ASL $nn,X
                let addr = (self.get_addr_zero_page() + self.x as usize) & 0xff;
                self.memory[addr] = self._asl_inst(self.memory[addr]);
                self.pc += 1;
            }

            // SHIFT LSR
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

            // SHIFT ROL - Rotate Left
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

            // SHIFT ROR - Rotate Right
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
    // LOAD
    //
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
    // NOP
    //
    #[test]
    fn test_nop() {
        let mut cpu = Cpu::new();
        cpu.run_opcode(&[NOP_EA]);

        let fresh = Cpu::new();
        assert_eq!(cpu, fresh);
    }

    // immediate, 1 byte instructions
    // #[test]
    fn test_inc() {
        // let mut cpu = Cpu::new();
        // cpu.run_opcode()
        //
    }

    fn test_dec() {
        //
    }

    //
    // INCREMENT
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
}
