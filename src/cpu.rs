#[derive(Debug, PartialEq)]
pub(crate) struct Cpu {
    a: u8,   // accumulator
    x: u8,   // x index register
    y: u8,   // y index register
    pc: u16, // program counter
    s: u8,   // stack
    p: u8,   // flags

    memory: [u8; 65536], // Silly, but currently memory is part of the processor
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
    pub const LDA: u8 = 0x00;
    pub const LDX: u8 = 0x00;
    pub const LDY: u8 = 0x00;
    pub const STA: u8 = 0x00;
    pub const STX: u8 = 0x00;
    pub const STY: u8 = 0x00;

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

            memory: [0; 65536],
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
        self.pc = self.get_addr();
    }

    fn get_addr(&mut self) -> u16 {
        let mut addr: u16 = self.memory[self.pc as usize] as u16;
        addr |= (self.memory[self.pc as usize + 1] as u16) << 8;
        addr
    }

    fn get_addr_zero_page(&mut self) -> u16 {
        self.memory[self.pc as usize] as u16
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
                // ASL A // accubulator
                self.a = self._asl_inst(self.a);

                //self.update_carry(self.a & 0x80 != 0);

                //self.a <<= 1;
                //self.update_negative(self.a & 0x80 != 0);
                //self.update_zero(self.a == 0);
            }
            opcodes::ASL_0E => {
                // ASL $nnnn // absolute
                let addr = self.get_addr();

                self.memory[addr as usize] = self._asl_inst(
                    self.memory[addr as usize]
                );
                //let mut val = self.memory[addr as usize];
                //self.update_carry(val & 0x80 != 0);
                //val <<= 1;
                //self.update_negative(val & 0x80 != 0);
                //self.update_zero(val == 0);
                //self.memory[addr as usize] = val;

                self.pc += 2;
            }
            opcodes::ASL_1E => {
                // ASL $nnnn,X // X-indexed absolute
                let mut addr = self.get_addr();
                addr += self.x as u16;

                self.memory[addr as usize] = self._asl_inst(
                    self.memory[addr as usize]
                );
                //let mut val = self.memory[addr as usize];
                //self.update_carry(val & 0x80 != 0);
                //val <<= 1;
                //self.update_negative(val & 0x80 != 0);
                //self.update_zero(val == 0);
                //self.memory[addr as usize] = val;

                self.pc += 2;
            }
            opcodes::ASL_06 => {
                // ASL $nn // zero page
                let addr = self.get_addr_zero_page();

                self.memory[addr as usize] = self._asl_inst(
                    self.memory[addr as usize]
                );
                // let mut val = self.memory[addr as usize];
                // self.update_carry(val & 0x80 != 0);
                // val <<= 1;
                // self.update_negative(val & 0x80 != 0);
                // self.update_zero(val == 0);
                // self.memory[addr as usize] = val;

                self.pc += 1;
            }
            opcodes::ASL_16 => {
                // ASL $nn,X // X-indexed zero page
                let mut addr = self.get_addr_zero_page();
                addr += self.x as u16;
                addr &= 0xff;

                self.memory[addr as usize] = self._asl_inst(
                    self.memory[addr as usize]
                );
                // let mut val = self.memory[addr as usize];
                // self.update_carry(val & 0x80 != 0);
                // val <<= 1;
                // self.update_negative(val & 0x80 != 0);
                // self.update_zero(val == 0);
                // self.memory[addr as usize] = val;

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

                self.memory[addr as usize] = self._lsr_inst(
                    self.memory[addr as usize]
                );

                self.pc += 2;
            }
            opcodes::LSR_5E => {
                // LSR $nnnn,X
                let mut addr = self.get_addr();
                addr += self.x as u16;

                self.memory[addr as usize] = self._lsr_inst(
                    self.memory[addr as usize]
                );

                self.pc += 2;
            }
            opcodes::LSR_46 => {
                // LSR $nn
                let addr = self.get_addr_zero_page();

                self.memory[addr as usize] = self._lsr_inst(
                    self.memory[addr as usize]
                );

                self.pc += 1;
            }
            opcodes::LSR_56 => {
                // LSR $nn,X
                let mut addr = self.get_addr_zero_page();
                addr += self.x as u16;
                addr &= 0xff;

                self.memory[addr as usize] = self._lsr_inst(
                    self.memory[addr as usize]
                );

                self.pc += 1;
            }

            // SHIFT ROL - Rotate Left
            opcodes::ROL_2A => {
                // ROL A
                self.a = self._rol_inst(self.a);
            }
            opcodes::ROL_2E => {
                // ROL $nnnn
                let addr: usize = self.get_addr() as usize;
                self.memory[addr] = self._rol_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::ROL_3E => {
                // ROL $nnnn,X
                let mut addr: usize = self.get_addr() as usize;
                addr += self.x as usize;

                self.memory[addr] = self._rol_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::ROL_26 => {
                // ROL $nn
                let addr: usize = self.get_addr_zero_page() as usize;
                self.memory[addr] = self._rol_inst(self.memory[addr]);
                self.pc += 1;
            }
            opcodes::ROL_36 => {
                // ROL $nn,X
                let mut addr: usize = self.get_addr_zero_page() as usize;
                addr += self.x as usize;
                addr &= 0xff;

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
                let addr: usize = self.get_addr() as usize;
                self.memory[addr] = self._ror_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::ROR_7E => {
                // ROR $nnnn,X
                let mut addr: usize = self.get_addr() as usize;
                addr += self.x as usize;

                self.memory[addr] = self._ror_inst(self.memory[addr]);
                self.pc += 2;
            }
            opcodes::ROR_66 => {
                // ROR $nn
                let addr: usize = self.get_addr_zero_page() as usize;
                self.memory[addr] = self._ror_inst(self.memory[addr]);
                self.pc += 1;
            }
            opcodes::ROR_76 => {
                // ROR $nn,X
                let mut addr: usize = self.get_addr_zero_page() as usize;
                addr += self.x as usize;
                addr &= 0xff;

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
    fn test_lsr_4a() { // LSR A
        let memory: &[u8] = &[LSR_4A];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.a = 0;
        cpu.step();
        assert!(cpu.p == Z_Zero);

        cpu.reset();
        cpu.a = 1;
        cpu.step();
        assert!(cpu.p == C_Carry | Z_Zero);
        assert!(cpu.a == 0x00);

        cpu.reset();
        cpu.a = 2;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.a == 1);

        cpu.reset();
        cpu.a = 3;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.a == 0x01);

        cpu.reset();
        cpu.a = 0x80;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.a == 0x40);

        cpu.reset();
        cpu.a = 0x81;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.a == 0x40);
    }

    #[test]
    fn test_lsr_4e() { // LSR $nnnn // absolute
        let memory: &[u8] = &[LSR_4E, 0x00, 0x80];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8000] = 0;
        cpu.step();
        assert!(cpu.p == Z_Zero);
        assert!(cpu.memory[0x8000] == 0x00);

        cpu.reset();
        cpu.memory[0x8000] = 1;
        cpu.step();
        assert!(cpu.p == C_Carry | Z_Zero);
        assert!(cpu.memory[0x8000] == 0);

        cpu.reset();
        cpu.memory[0x8000] = 2;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x8000] == 1);

        cpu.reset();
        cpu.memory[0x8000] = 3;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.memory[0x8000] == 1);

        cpu.reset();
        cpu.memory[0x8000] = 0x80;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x8000] == 0x40);

        cpu.reset();
        cpu.memory[0x8000] = 0x81;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.memory[0x8000] == 0x40);
    }

    #[test]
    fn test_lsr_5e() { // LSR $nnnn,X
        let memory: &[u8] = &[LSR_5E, 0x00, 0x80];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x8015] = 0;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == Z_Zero);
        assert!(cpu.memory[0x8015] == 0x00);

        cpu.reset();
        cpu.memory[0x8015] = 1;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == C_Carry | Z_Zero);
        assert!(cpu.memory[0x8015] == 0);

        cpu.reset();
        cpu.memory[0x8015] = 2;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x8015] == 1);

        cpu.reset();
        cpu.memory[0x8015] = 3;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.memory[0x8015] == 1);

        cpu.reset();
        cpu.memory[0x8015] = 0x80;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x8015] == 0x40);

        cpu.reset();
        cpu.memory[0x8015] = 0x81;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.memory[0x8015] == 0x40);
    }

    #[test]
    fn test_lsr_46() { // LSR $nn // zero-page
        let memory: &[u8] = &[LSR_46, 0x80];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0080] = 0;
        cpu.step();
        assert!(cpu.p == Z_Zero);
        assert!(cpu.memory[0x0080] == 0x00);

        cpu.reset();
        cpu.memory[0x0080] = 1;
        cpu.step();
        assert!(cpu.p == C_Carry | Z_Zero);
        assert!(cpu.memory[0x0080] == 0);

        cpu.reset();
        cpu.memory[0x0080] = 2;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x0080] == 1);

        cpu.reset();
        cpu.memory[0x0080] = 3;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.memory[0x0080] == 1);

        cpu.reset();
        cpu.memory[0x0080] = 0x80;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x0080] == 0x40);

        cpu.reset();
        cpu.memory[0x0080] = 0x81;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.memory[0x0080] == 0x40);
    }

    #[test]
    fn test_lsr_56() { // LSR $nn,X // X-indexed zero-page
        let memory: &[u8] = &[LSR_56, 0x80];

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, memory);
        cpu.memory[0x0095] = 0;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == Z_Zero);
        assert!(cpu.memory[0x0095] == 0x00);

        cpu.reset();
        cpu.memory[0x0095] = 1;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == C_Carry | Z_Zero);
        assert!(cpu.memory[0x0095] == 0);

        cpu.reset();
        cpu.memory[0x0095] = 2;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x0095] == 1);

        cpu.reset();
        cpu.memory[0x0095] = 3;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.memory[0x0095] == 1);

        cpu.reset();
        cpu.memory[0x0095] = 0x80;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x0095] == 0x40);

        cpu.reset();
        cpu.memory[0x0095] = 0x81;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.memory[0x0095] == 0x40);
    }

    // ROL - Rotate Left
    #[test]
    fn test_rol_2a() {
        let memory: &[u8] = &[ROL_2A]; // ROL A

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.a = 0;
        cpu.step();
        assert!(cpu.p == Z_Zero);
        assert!(cpu.a == 0);

        cpu.reset();
        cpu.a = 1;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.a == 2);

        cpu.reset();
        cpu.a = 0x40;
        cpu.step();
        assert!(cpu.p == N_Negative);
        assert!(cpu.a == 0x80);

        cpu.reset();
        cpu.a = 0x80;
        cpu.step();
        assert!(cpu.p == C_Carry | Z_Zero);
        assert!(cpu.a == 0);

        cpu.reset();
        cpu.a = 0xC0;
        cpu.step();
        assert!(cpu.p == C_Carry | N_Negative);
        assert!(cpu.a == 0x80);

        cpu.reset();
        cpu.update_carry(true);
        cpu.a = 0x40;
        cpu.step();
        assert!(cpu.p == N_Negative);
        assert!(cpu.a == 0x81);

        cpu.reset();
        cpu.update_carry(true);
        cpu.a = 0x80;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.a == 0x01);

        cpu.reset();
        cpu.update_carry(true);
        cpu.a = 0xC0;
        cpu.step();
        assert!(cpu.p == C_Carry | N_Negative);
        assert!(cpu.a == 0x81);
    }

    #[test]
    fn test_rol_2e() {
        let memory: &[u8] = &[ROL_2E, 0x00, 0x80]; // ROL $8000

        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.memory[0x8000] = 0;
        cpu.step();
        assert!(cpu.p == Z_Zero);
        assert!(cpu.memory[0x8000] == 0);

        cpu.reset();
        cpu.memory[0x8000] = 1;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x8000] == 2);

        cpu.reset();
        cpu.memory[0x8000] = 0x40;
        cpu.step();
        assert!(cpu.p == N_Negative);
        assert!(cpu.memory[0x8000] == 0x80);

        cpu.reset();
        cpu.memory[0x8000] = 0x80;
        cpu.step();
        assert!(cpu.p == C_Carry | Z_Zero);
        assert!(cpu.memory[0x8000] == 0);

        cpu.reset();
        cpu.memory[0x8000] = 0xC0;
        cpu.step();
        assert!(cpu.p == C_Carry | N_Negative);
        assert!(cpu.memory[0x8000] == 0x80);

        cpu.reset();
        cpu.update_carry(true);
        cpu.memory[0x8000] = 0x40;
        cpu.step();
        assert!(cpu.p == N_Negative);
        assert!(cpu.memory[0x8000] == 0x81);

        cpu.reset();
        cpu.update_carry(true);
        cpu.memory[0x8000] = 0x80;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.memory[0x8000] == 0x01);

        cpu.reset();
        cpu.update_carry(true);
        cpu.memory[0x8000] = 0xC0;
        cpu.step();
        assert!(cpu.p == C_Carry | N_Negative);
        assert!(cpu.memory[0x8000] == 0x81);
    }

    #[test]
    fn test_rol_3e() {
        let memory: &[u8] = &[ROL_3E, 0x00, 0x80]; // ROL $8000,X

        fn _t(memory: &[u8], addr: u16, x: u8, set_carry: bool,
              val: u8, expected_val: u8, expected_flags: u8)
        {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.memory[addr as usize + x as usize] = val;
            cpu.x = x;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize + x as usize] == expected_val);
        }

        // memory    addr     x  carry  val exp_val,   exp_flags
        _t(memory, 0x8000, 0x15, false,    0,      0,     Z_Zero);
        _t(memory, 0x8000, 0x15, false,    1,      2,          0);
        _t(memory, 0x8000, 0x15, false, 0x40,   0x80, N_Negative);
        _t(memory, 0x8000, 0x15, false, 0x80,      0,    C_Carry | Z_Zero);
        _t(memory, 0x8000, 0x15, false, 0xC0,   0x80, C_Carry | N_Negative);
        _t(memory, 0x8000, 0x15,  true, 0x40,   0x81, N_Negative);
        _t(memory, 0x8000, 0x15,  true, 0x80,      1,    C_Carry);
        _t(memory, 0x8000, 0x15,  true, 0xC0,   0x81, N_Negative | C_Carry);

        /*
        let mut cpu = Cpu::new();
        cpu.patch_memory(0, &memory);
        cpu.memory[0x8015] = 0;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == Z_Zero);
        assert!(cpu.memory[0x8015], 0);

        cpu.reset();
        cpu.memory[0x8015] = 1;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == 0);
        assert!(cpu.memory[0x8015] == 2);

        cpu.reset();
        cpu.memory[0x8015] = 0x40;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == N_Negative);
        assert!(cpu.memory[0x8015] == 0x80);

        cpu.reset();
        cpu.memory[0x8015] = 0x80;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.memory[0x8015] == 0);

        cpu.reset();
        cpu.memory[0x8015] = 0xC0;
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == C_Carry | N_Negative);
        assert!(cpu.memory[0x8015] == 0x80);

        cpu.reset();
        cpu.memory[0x8015] = 0x40;
        cpu.update_carry(true);
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == N_Negative);
        assert!(cpu.memory[0x8015] == 0x81);

        cpu.reset();
        cpu.memory[0x8015] = 0x80;
        cpu.update_carry(true);
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == C_Carry);
        assert!(cpu.memory[0x8015] == 0x01);

        cpu.reset();
        cpu.memory[0x8015] = 0xC0;
        cpu.update_carry(true);
        cpu.x = 0x15;
        cpu.step();
        assert!(cpu.p == C_Carry | N_Negative);
        assert!(cpu.memory[0x8015] == 0x81);
        */
    }

    #[test]
    fn test_rol_26() {
        // ROL $nn
        let memory: &[u8] = &[ROL_26, 0x80]; // ROL $80

        fn _t(memory: &[u8], addr: u16, set_carry: bool,
              val: u8, expected_val: u8, expected_flags: u8)
        {
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
        _t(memory, 0x0080, false,    0,      0,     Z_Zero);
        _t(memory, 0x0080, false,    1,      2,          0);
        _t(memory, 0x0080, false, 0x40,   0x80, N_Negative);
        _t(memory, 0x0080, false, 0x80,      0,    C_Carry | Z_Zero);
        _t(memory, 0x0080, false, 0xC0,   0x80, C_Carry | N_Negative);
        _t(memory, 0x0080,  true, 0x40,   0x81, N_Negative);
        _t(memory, 0x0080,  true, 0x80,      1,    C_Carry);
        _t(memory, 0x0080,  true, 0xC0,   0x81, N_Negative | C_Carry);
    }

    #[test]
    fn test_rol_36() {
        let memory: &[u8] = &[ROL_36, 0x80]; // ROL $80,X

        fn _t(memory: &[u8], addr: u16, x: u8, set_carry: bool,
              val: u8, expected_val: u8, expected_flags: u8)
        {
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
        _t(memory, 0x0080, 0x15, false,    0,      0,     Z_Zero);
        _t(memory, 0x0080, 0x15, false,    1,      2,          0);
        _t(memory, 0x0080, 0x15, false, 0x40,   0x80, N_Negative);
        _t(memory, 0x0080, 0x15, false, 0x80,      0,    C_Carry | Z_Zero);
        _t(memory, 0x0080, 0x15, false, 0xC0,   0x80, C_Carry | N_Negative);
        _t(memory, 0x0080, 0x15,  true, 0x40,   0x81, N_Negative);
        _t(memory, 0x0080, 0x15,  true, 0x80,      1,    C_Carry);
        _t(memory, 0x0080, 0x15,  true, 0xC0,   0x81, N_Negative | C_Carry);
    }

    // ROR - Rotate Right
    #[test]
    fn test_ror_6A() {
        let memory: &[u8] = &[ROR_6A]; // ROR A

        fn _t(memory: &[u8], set_carry: bool,
              val: u8, expected_val: u8, expected_flags: u8)
        {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.a = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.a == expected_val);
        }

        // memory  carry  val  exp_val, exp_flags
        _t(memory, false,    0,      0, Z_Zero);
        _t(memory, false,    1,      0, C_Carry | Z_Zero);
        _t(memory, false,    2,      1, 0);
        _t(memory, false,    3,      1, C_Carry);
        _t(memory,  true,    0,   0x80, N_Negative);
        _t(memory,  true,    1,   0x80, N_Negative | C_Carry);
        _t(memory,  true,    2,   0x81, N_Negative);
        _t(memory,  true,    3,   0x81, N_Negative | C_Carry);
        _t(memory,  true, 0x80,   0xC0, N_Negative);
        _t(memory,  true, 0x81,   0xC0, N_Negative | C_Carry);
    }

    #[test]
    fn test_ror_6E() {
        let memory: &[u8] = &[ROR_6E, 0x40, 0x80]; // ROR $nnnn


        fn _t(memory: &[u8], addr: u16, set_carry: bool,
              val: u8, expected_val: u8, expected_flags: u8)
        {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.memory[addr as usize] = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize] == expected_val);
        }

        // memory    addr, carry   val exp_val,  exp_flags
        _t(memory, 0x8040, false,    0,      0,     Z_Zero);
        _t(memory, 0x8040, false,    1,      0, C_Carry | Z_Zero);
        _t(memory, 0x8040, false,    2,      1,          0);
        _t(memory, 0x8040, false,    3,      1,    C_Carry);
        _t(memory, 0x8040,  true,    0,   0x80, N_Negative);
        _t(memory, 0x8040,  true,    1,   0x80, N_Negative | C_Carry);
        _t(memory, 0x8040,  true,    2,   0x81, N_Negative);
        _t(memory, 0x8040,  true,    3,   0x81, N_Negative | C_Carry);
        _t(memory, 0x8040,  true, 0x80,   0xC0, N_Negative);
        _t(memory, 0x8040,  true, 0x81,   0xC0, N_Negative | C_Carry);
    }

    #[test]
    fn test_ror_7E() {
        let memory: &[u8] = &[ROR_7E, 0x40, 0x80]; // ROR $nnnn,X

        fn _t(memory: &[u8], addr: u16, x: u8, set_carry: bool,
              val: u8, expected_val: u8, expected_flags: u8)
        {
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
        _t(memory, 0x8040, 0x20, false,    0,      0,     Z_Zero);
        _t(memory, 0x8040, 0x20, false,    1,      0, C_Carry | Z_Zero);
        _t(memory, 0x8040, 0x20, false,    2,      1,          0);
        _t(memory, 0x8040, 0x20, false,    3,      1,    C_Carry);
        _t(memory, 0x8040, 0x20,  true,    0,   0x80, N_Negative);
        _t(memory, 0x8040, 0x20,  true,    1,   0x80, N_Negative | C_Carry);
        _t(memory, 0x8040, 0x20,  true,    2,   0x81, N_Negative);
        _t(memory, 0x8040, 0x20,  true,    3,   0x81, N_Negative | C_Carry);
        _t(memory, 0x8040, 0x20,  true, 0x80,   0xC0, N_Negative);
        _t(memory, 0x8040, 0x20,  true, 0x81,   0xC0, N_Negative | C_Carry);
    }

    #[test]
    fn test_ror_66() {
        let memory: &[u8] = &[ROR_66, 0x40]; // ROR $nn

        fn _t(memory: &[u8], addr: u16, set_carry: bool,
              val: u8, expected_val: u8, expected_flags: u8)
        {
            let mut cpu = Cpu::new();
            cpu.patch_memory(0, memory);
            cpu.update_carry(set_carry);
            cpu.memory[addr as usize] = val;
            cpu.step();

            assert!(cpu.p == expected_flags);
            assert!(cpu.memory[addr as usize] == expected_val);
        }

        // memory    addr, carry   val exp_val,  exp_flags
        _t(memory, 0x0040, false,    0,      0,     Z_Zero);
        _t(memory, 0x0040, false,    1,      0, C_Carry | Z_Zero);
        _t(memory, 0x0040, false,    2,      1,          0);
        _t(memory, 0x0040, false,    3,      1,    C_Carry);
        _t(memory, 0x0040,  true,    0,   0x80, N_Negative);
        _t(memory, 0x0040,  true,    1,   0x80, N_Negative | C_Carry);
        _t(memory, 0x0040,  true,    2,   0x81, N_Negative);
        _t(memory, 0x0040,  true,    3,   0x81, N_Negative | C_Carry);
        _t(memory, 0x0040,  true, 0x80,   0xC0, N_Negative);
        _t(memory, 0x0040,  true, 0x81,   0xC0, N_Negative | C_Carry);
    }

    #[test]
    fn test_ror_76() {
        let memory: &[u8] = &[ROR_76, 0x40]; // ROR $nn,X

        fn _t(memory: &[u8], addr: u16, x: u8, set_carry: bool,
              val: u8, expected_val: u8, expected_flags: u8)
        {
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
        _t(memory, 0x0040, 0x20, false,    0,      0,     Z_Zero);
        _t(memory, 0x0040, 0x20, false,    1,      0, C_Carry | Z_Zero);
        _t(memory, 0x0040, 0x20, false,    2,      1,          0);
        _t(memory, 0x0040, 0x20, false,    3,      1,    C_Carry);
        _t(memory, 0x0040, 0x20,  true,    0,   0x80, N_Negative);
        _t(memory, 0x0040, 0x20,  true,    1,   0x80, N_Negative | C_Carry);
        _t(memory, 0x0040, 0x20,  true,    2,   0x81, N_Negative);
        _t(memory, 0x0040, 0x20,  true,    3,   0x81, N_Negative | C_Carry);
        _t(memory, 0x0040, 0x20,  true, 0x80,   0xC0, N_Negative);
        _t(memory, 0x0040, 0x20,  true, 0x81,   0xC0, N_Negative | C_Carry);
    }
}
