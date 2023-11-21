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

mod opcodes {
    // nop
    pub const NOP_EA: u8 = 0xEA;

    // increment
    pub const INX_E8: u8 = 0xE8;
    pub const INY_C8: u8 = 0xC8;
    pub const DEX_CA: u8 = 0xCA;
    pub const DEY_88: u8 = 0x88;

    // flags
    pub const CLC_18: u8 = 0x18;
    pub const CLD_D8: u8 = 0xD8;
    pub const CLI_58: u8 = 0x58;
    pub const CLV_B8: u8 = 0xB8;
    pub const SEC_38: u8 = 0x38;
    pub const SED_F8: u8 = 0xF8;
    pub const SEI_78: u8 = 0x78;

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

    fn update_negative(&mut self, val: u8) {
        if val & Flags::N_Negative != 0 {
            self.p |= Flags::N_Negative;
        } else {
            self.p &= !Flags::N_Negative;
        }
    }

    fn update_zero(&mut self, val: u8) {
        if val == 0 {
            self.p |= Flags::Z_Zero;
        } else {
            self.p &= !Flags::Z_Zero;
        }
    }

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
                self.update_negative(self.x);
                self.update_zero(self.x);
            }
            opcodes::TAY_A8 => {
                self.y = self.a;
                self.update_negative(self.y);
                self.update_zero(self.y);
            }
            opcodes::TSX_BA => {
                self.x = self.s;
                self.update_negative(self.x);
                self.update_zero(self.x);
            }
            opcodes::TXA_8A => {
                self.a = self.x;
                self.update_negative(self.a);
                self.update_zero(self.a);
            }
            opcodes::TXS_9A => {
                self.s = self.x;
                self.update_negative(self.s);
                self.update_zero(self.s);
            }
            opcodes::TYA_98 => {
                self.a = self.y;
                self.update_negative(self.a);
                self.update_zero(self.a);
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
    use super::opcodes::*;
    use super::*;

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
}
