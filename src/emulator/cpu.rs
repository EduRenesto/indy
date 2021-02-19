//! Implementação da própria CPU MIPS.

use super::Instruction;
use super::Register;
use super::Memory;

use super::instr::sign_extend;

use color_eyre::eyre::{Result, eyre};

struct Registers([u32; 32]);

impl std::ops::Index<Register> for Registers {
    type Output = u32;

    fn index(&self, index: Register) -> &Self::Output {
        if index.0 == 0 {
            &0
        } else {
            &self.0[index.0 as usize]
        }
    }
}

impl std::ops::IndexMut<Register> for Registers {
    fn index_mut(&mut self, index: Register) -> &mut Self::Output {
        // Podemos "escrever" no $zero, mas como toda read retorna 0,
        // tudo bem.
        // Não vou descartar escrita no $zero porque assim fica dentro
        // das normas de safety da linguagem.
        // Antes eu tinha colocado um unreachable!() na escrita em zero,
        // mas não dá pra garantir que nunca existirá uma escrita no zero.
        &mut self.0[index.0 as usize]
    }
}

impl std::fmt::Display for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for i in 0..32 {
            writeln!(f, "{}: {:#x}", Register(i as u32), &self.0[i])?;
        }

        Ok(())
    }
}

/// Essa struct encapsula o estado da CPU, assim como a instância da memória.
pub struct Cpu {
    /// 31 registradores de 32 bits.
    /// Na verdade, a ISA MIPS especifica 32 regs de 32 bits, no entanto não
    /// precisamos implementar o reg $zero.
    regs: Registers,

    /// A instância da memória ligada a CPU atual.
    mem: Memory,

    /// O program counter.
    pc: u32,

    /// A CPU terminou a execução?
    halt: bool,
}

impl Cpu {
    pub fn new(start: u32) -> Cpu {
        // TODO set gp, sp
        Cpu {
            regs: Registers([0; 32]),
            mem: Memory::new(),
            pc: start,
            halt: false,
        }
    }

    pub fn memory(&self) -> &Memory {
        &self.mem
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mem
    }

    pub fn cycle(&mut self) -> Result<()> {
        let word = self.mem.peek(self.pc)?;

        let instr = Instruction::decode(*word)?;
        //println!("{}", instr);
        match instr {
            Instruction::ADD(args) => {
                self.regs[args.rd] = self.regs[args.rs] + self.regs[args.rt];
            },
            Instruction::ADDI(args) => {
                self.regs[args.rt] = self.regs[args.rs] + sign_extend(args.imm, 16);
            },
            Instruction::SYSCALL => {
                match self.regs[Register(2)] {
                    1 => {
                        print!("syscall: {}", self.regs[Register(4)]);
                    },
                    4 => {
                        let mut addr = self.regs[Register(4)];
                        println!("syscall: string at {:#x}", addr);

                        let mut val = self.mem.peek_unaligned(addr)?;

                        while val != 0 {
                            print!("{}", val as char);
                            addr += 1;
                            val = self.mem.peek_unaligned(addr)?;
                        }
                    },
                    5 => {
                        println!("syscall: TODO read integer");
                    },
                    10 => {
                        self.halt = true;
                        println!("syscall: halted");
                    },
                    11 => {
                        print!("syscall: {}", self.regs[Register(4)] as u8 as char);
                    },
                    a => println!("syscall: unknown syscall {}", a)
                };
            },
            Instruction::LUI(args) => {
                let val = args.imm << (32 - 16);
                self.regs[args.rt] = val;
            },
            Instruction::ORI(args) => {
                self.regs[args.rt] = self.regs[args.rs] | args.imm;
            },
            Instruction::ADDIU(args) => {
                self.regs[args.rt] = self.regs[args.rs].overflowing_add(args.imm).0;
            }
            a => return Err(eyre!("Instruction {} not implemented yet!", a)),
        }

        self.pc += 4;

        Ok(())
    }

    pub fn run(&mut self) -> Result<()> {
        while !self.halt {
            self.cycle()?;
        }

        Ok(())
    }
}
