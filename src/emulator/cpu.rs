//! Implementação da própria CPU MIPS.

use super::Instruction;
use super::Memory;
use super::Register;

use super::instr::{branch_addr, jump_addr, sign_extend, sign_extend_cast};

use std::io::Write;

use color_eyre::eyre::{eyre, Result};

struct Registers([u32; 32]);

fn as_signed(val: u32) -> i32 {
    i32::from_le_bytes(val.to_le_bytes())
}

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
    pub fn new(start: u32, sp: u32, gp: u32) -> Cpu {
        // TODO set gp, sp
        let mut cpu = Cpu {
            regs: Registers([0; 32]),
            mem: Memory::new(),
            pc: start,
            halt: false,
        };

        cpu.regs[Register(28)] = gp;
        cpu.regs[Register(29)] = sp;

        cpu
    }

    #[allow(dead_code)]
    pub fn memory(&self) -> &Memory {
        &self.mem
    }

    pub fn memory_mut(&mut self) -> &mut Memory {
        &mut self.mem
    }

    #[allow(unreachable_patterns)]
    pub fn cycle(&mut self) -> Result<()> {
        let word = self.mem.peek(self.pc)?;

        let instr = Instruction::decode(*word)?;
        //println!("{:#010x}: {}", self.pc, instr);
        match instr {
            Instruction::ADD(args) => {
                //self.regs[args.rd] = self.regs[args.rs] + self.regs[args.rt];
                self.regs[args.rd] = self.regs[args.rs].overflowing_add(self.regs[args.rt]).0;
            }
            Instruction::ADDI(args) => {
                //self.regs[args.rt] = self.regs[args.rs] + sign_extend(args.imm, 16);
                self.regs[args.rt] = self.regs[args.rs]
                    .overflowing_add(sign_extend(args.imm, 16))
                    .0;
            }
            Instruction::SYSCALL(_) => {
                //println!("debug: syscall");
                //println!("{}", self.regs);
                match self.regs[Register(2)] {
                    1 => {
                        print!("{}", self.regs[Register(4)]);
                    }
                    4 => {
                        let mut addr = self.regs[Register(4)];
                        //println!("syscall: string at {:#x}", addr);

                        let mut val = self.mem.peek_unaligned(addr)?;

                        while val != 0 {
                            print!("{}", val as char);
                            addr += 1;
                            val = self.mem.peek_unaligned(addr)?;
                        }
                    }
                    5 => {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;

                        let val = input.trim().parse::<u32>()?;

                        self.regs[Register(2)] = val;
                    }
                    10 => {
                        self.halt = true;
                        println!("syscall: halted");
                    }
                    11 => {
                        print!("{}", self.regs[Register(4)] as u8 as char);
                    }
                    a => println!("syscall: unknown syscall {}", a),
                };
                // zsh dentro do term-mode do emacs faz com que o stdout bugue (????)
                // então temos que flushar o stdout quando termina a syscall.
                //
                // ...acho que é uma desculpa pra eu perder tempo configurando
                // exwm e usar o alacritty direito!
                std::io::stdout().flush()?;
            }
            Instruction::LUI(args) => {
                let val = args.imm << (32 - 16);
                self.regs[args.rt] = val;
            }
            Instruction::ORI(args) => {
                self.regs[args.rt] = self.regs[args.rs] | args.imm;
            }
            Instruction::ADDIU(args) => {
                //self.regs[args.rt] = self.regs[args.rs].overflowing_add(args.imm).0;
                self.regs[args.rt] = self.regs[args.rs]
                    .overflowing_add(sign_extend(args.imm, 16))
                    .0;
            }
            Instruction::ADDU(args) => {
                self.regs[args.rd] = self.regs[args.rs] + self.regs[args.rt];
            }
            Instruction::BEQ(args) => {
                if self.regs[args.rs] == self.regs[args.rt] {
                    let target = branch_addr(args.imm);
                    //println!("jump to {:#x}, pc is {:#x}", target, self.pc);
                    //println!("target % 4 = {}", target % 4);
                    self.pc = (self.pc as i32 + target) as u32;
                }
            }
            Instruction::BNE(args) => {
                //println!("jump to {}, pc is {}", args.imm, self.pc);
                if self.regs[args.rs] != self.regs[args.rt] {
                    let target = branch_addr(args.imm);
                    //println!("jump to {:#x}, pc is {:#x}", target, self.pc);
                    self.pc = (self.pc as i32 + target) as u32;
                }
            }
            Instruction::J(addr) => {
                let target = jump_addr(self.pc, addr);
                //println!("jumping to {:#010x} which is {:#010x}", addr, target);
                self.pc = target - 4;
            }
            Instruction::SLT(args) => {
                //println!("SLT: {} < {}?", as_signed(self.regs[args.rs]), as_signed(self.regs[args.rt]));
                self.regs[args.rd] =
                    if as_signed(self.regs[args.rs]) < as_signed(self.regs[args.rt]) {
                        1
                    } else {
                        0
                    };
            }
            Instruction::JR(args) => {
                self.pc = self.regs[args.rs];
            }
            Instruction::JAL(addr) => {
                self.regs[Register(31)] = self.pc;
                let target = jump_addr(self.pc, addr);
                //println!("jumping to {:#010x} which is {:#010x}", addr, target);
                self.pc = target - 4;
            }
            Instruction::SLL(args) => {
                self.regs[args.rd] = self.regs[args.rt] << args.shamt;
            }
            Instruction::SRL(args) => {
                self.regs[args.rd] = self.regs[args.rt] >> args.shamt;
            }
            Instruction::ANDI(args) => {
                self.regs[args.rt] = self.regs[args.rs] & args.imm;
            }
            Instruction::LW(args) => {
                let addr = self.regs[args.rs] as i32 + sign_extend_cast(args.imm, 16);
                self.regs[args.rt] = *self.mem.peek(addr as u32)?;
            }
            Instruction::SW(args) => {
                let addr = self.regs[args.rs] as i32 + sign_extend_cast(args.imm, 16);
                self.mem.poke(addr as u32, self.regs[args.rt])?;
            }
            Instruction::OR(args) => {
                self.regs[args.rd] = self.regs[args.rs] | self.regs[args.rt];
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
