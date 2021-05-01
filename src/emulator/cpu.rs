//! Implementação da própria CPU MIPS. Aqui são interpretadas as instruções.
//!
//! Note que um tema comum nesse arquivo é a utilização do idiom `newtype`
//! (mesma ideia que em Haskell :p).

use super::memory::Memory;
use super::FloatRegister;
use super::Instruction;
use super::Register;

use super::instr::{branch_addr, jump_addr, sign_extend, sign_extend_cast};

use super::stats::StatsReporter;

use std::cell::UnsafeCell;
use std::convert::TryInto;
use std::io::Write;

use color_eyre::eyre::{eyre, Result};

use log::debug;

/// Nosso processador MIPS tem 32 registradores de 32 bits.
/// Essa newtype encapsula uma array de 32*u32 para que possamos implementar
/// traits arbitrários como quisermos nela, além de melhorar a legibilidade.
struct Registers([u32; 32]);

/// Armazena os 32 registradores de ponto flutuante.
///
/// Não armazenamos diretamente os f32 porque assim fica mais fácil de
/// fazer as conversões single <-> double.
struct FloatRegisters([u32; 32]);

/// Reinterpreta os bits de um unsigned de 32 bits como um signed de 32 bits.
fn as_signed(val: u32) -> i32 {
    //i32::from_le_bytes(val.to_le_bytes())
    unsafe {
        let ptr = (&val as *const u32) as *const i32;
        *ptr
    }
}

/// Reinterpreta os bits de um signed de 32 bits como um unsigned de 32 bits.
fn as_unsigned(val: i32) -> u32 {
    //u32::from_le_bytes(val.to_le_bytes())
    unsafe {
        let ptr = (&val as *const i32) as *const u32;
        *ptr
    }
}

/// Reinterpreta os bits de um unsigned de 32 bits como um float de single precision.
fn word_to_single(val: u32) -> f32 {
    // TODO does this work?
    // Se sim, devo fazer nos outros acima?
    // Isso é _no additional copy_, mas e se &val nao existir mais?
    // Provavelmente vai.
    unsafe {
        let ptr = (&val as *const u32) as *const f32;
        *ptr
    }
}

/// Reinterpreta os bits de dois unsigned de 32 bits como um float de double precision.
fn dword_to_double(lo: u32, hi: u32) -> f64 {
    let arr = [lo, hi];

    unsafe {
        let ptr = (arr.as_ptr() as *const u32) as *const f64;
        *ptr
    }

    //f64::from_le_bytes((lo as u64 | ((hi as u64) << 32)).to_le_bytes())
}

/// Reinterpreta os bits de um float de single precision como um unsigned de 32 bits.
fn single_to_word(val: f32) -> u32 {
    unsafe {
        let ptr = (&val as *const f32) as *const u32;
        *ptr
    }

    //u32::from_le_bytes(val.to_le_bytes())
}

/// Reinterpreta os bits de um float de double precision como dois unsigned de 32 bits.
fn double_to_dword(val: f64) -> (u32, u32) {
    unsafe {
        // Esse é o meu primeiro transmute útil.
        // Me sinto profissional :p
        let arr: [u32; 2] = std::mem::transmute(val);
        (arr[0], arr[1])
    }

    //let dword = u64::from_le_bytes(val.to_le_bytes());
    //((dword & 0xFFFFFFFF) as u32, ((dword & 0xFFFFFFFF00000000) >> 32) as u32)
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

impl std::ops::Index<FloatRegister> for FloatRegisters {
    type Output = u32;

    fn index(&self, index: FloatRegister) -> &Self::Output {
        &self.0[index.0 as usize]
    }
}

impl std::ops::IndexMut<FloatRegister> for FloatRegisters {
    fn index_mut(&mut self, index: FloatRegister) -> &mut Self::Output {
        &mut self.0[index.0 as usize]
    }
}

/// Essa struct encapsula o estado da CPU, assim como a instância da memória.
pub struct Cpu<'a, TD: Memory, TI: Memory> {
    /// 32 registradores de 32 bits.
    regs: Registers,

    /// A instância da memória de dados ligada a CPU atual.
    mem: &'a UnsafeCell<TD>,

    /// A instância da memória de instruções ligada a CPU atual.
    imem: &'a UnsafeCell<TI>,

    /// O program counter.
    pc: u32,

    /// É verdadeiro se a instrução atual está num branch delay slot.
    in_delay_slot: bool,

    /// Endereço para qual o branch pulará.
    branch_to: Option<u32>,

    /// A CPU terminou a execução?
    halt: bool,

    /// Os registradores de aritmetica.
    arith_regs: (u32, u32),

    /// 32 registradores de ponto flutuante.
    float_regs: FloatRegisters,

    /// Contador de estatísticas.
    stats: StatsReporter,

    /// Condition code do cop1. TODO implementar todos!
    float_cc: bool,
}

impl<'a, TD: Memory, TI: Memory> Cpu<'a, TD, TI> {
    /// Cria uma nova instância da CPU, colocando o program counter no
    /// endereço `start` especificado.
    pub fn new(
        mem: &'a UnsafeCell<TD>,
        imem: &'a UnsafeCell<TI>,
        start: u32,
        sp: u32,
        gp: u32,
    ) -> Self {
        let mut cpu = Cpu {
            regs: Registers([0; 32]),
            mem,
            imem,
            pc: start,
            in_delay_slot: false,
            branch_to: None,
            halt: false,
            arith_regs: (0, 0),
            float_regs: FloatRegisters([0; 32]),
            stats: StatsReporter::new(),
            float_cc: false,
        };

        cpu.regs[Register(28)] = gp;
        cpu.regs[Register(29)] = sp;

        cpu
    }

    /// Executa a instrução apontada pelo program counter atual. Retorna
    /// `Ok(())` se nenhum problema ocorreu.
    ///
    /// Aqui se encontram as implementações das instruções.
    #[allow(unreachable_patterns)]
    pub fn cycle(&mut self) -> Result<()> {
        match self.branch_to {
            Some(target) if target != self.pc => {
                self.in_delay_slot = true;
                //println!("Inside branch delay slot! Target: {:#010x}", target);
            }
            _ => {}
        }

        let (word, fetch_latency) = self.imem.peek_instruction(self.pc)?;

        let instr = Instruction::decode(word)?;
        self.stats.add_instr(&instr);
        self.stats.add_cycles(fetch_latency);
        debug!(
            "{:#010x}: {}; fetch: {} cycles",
            self.pc, instr, fetch_latency
        );
        match instr {
            Instruction::NOP => {
                self.stats.add_cycles(1);
            }
            Instruction::ADD(args) => {
                self.regs[args.rd] = self.regs[args.rs].overflowing_add(self.regs[args.rt]).0;
                self.stats.add_cycles(1);
            }
            Instruction::ADDI(args) => {
                self.regs[args.rt] = self.regs[args.rs]
                    .overflowing_add(sign_extend(args.imm, 16))
                    .0;
                self.stats.add_cycles(1);
            }
            Instruction::SYSCALL(_) => {
                //println!("debug: syscall");
                //println!("{}", self.regs);
                match self.regs[Register(2)] {
                    1 => {
                        print!("{}", as_signed(self.regs[Register(4)]));
                        self.stats.add_cycles(1);
                    }
                    2 => {
                        print!("{}", word_to_single(self.float_regs[FloatRegister(12)]));
                        self.stats.add_cycles(1);
                    }
                    3 => {
                        print!(
                            "{}",
                            dword_to_double(
                                self.float_regs[FloatRegister(12)],
                                self.float_regs[FloatRegister(13)]
                            )
                        );
                        self.stats.add_cycles(1);
                    }
                    4 => {
                        let mut addr = self.regs[Register(4)];
                        let mut byte_offset = addr % 4;
                        addr = addr - byte_offset;

                        let mut total_cycles = 0;

                        'outer: loop {
                            let (val, cycles) = self.mem.peek(addr)?;
                            let val = val.to_le_bytes();

                            //if cycles > total_cycles {
                            //    total_cycles = cycles;
                            //}
                            self.stats.add_cycles(cycles);

                            for i in byte_offset..4 {
                                let c = val[i as usize];
                                if c != 0 {
                                    print!("{}", c as char)
                                } else {
                                    break 'outer;
                                }
                            }

                            addr += 4;
                            byte_offset = 0;
                        }

                        self.stats.add_cycles(total_cycles);
                    }
                    5 => {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;

                        let val = input.trim().parse::<i32>()?;

                        self.regs[Register(2)] = as_unsigned(val);

                        self.stats.add_cycles(1);
                    }
                    6 => {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;

                        let val = input.trim().parse::<f32>()?;

                        self.float_regs[FloatRegister(0)] = single_to_word(val);

                        self.stats.add_cycles(1);
                    }
                    7 => {
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input)?;

                        let val = input.trim().parse::<f64>()?;

                        let (lo, hi) = double_to_dword(val);

                        self.float_regs[FloatRegister(0)] = lo;
                        self.float_regs[FloatRegister(1)] = hi;
                        self.stats.add_cycles(1);
                    }
                    10 => {
                        self.halt = true;
                        self.stats.add_cycles(1);
                        self.stats.finish();
                    }
                    11 => {
                        print!("{}", self.regs[Register(4)] as u8 as char);
                        self.stats.add_cycles(1);
                    }
                    500 => {
                        self.mem.dump()?;
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
                self.stats.add_cycles(1);
            }
            Instruction::ORI(args) => {
                self.regs[args.rt] = self.regs[args.rs] | args.imm;
                self.stats.add_cycles(1);
            }
            Instruction::ADDIU(args) => {
                self.regs[args.rt] = self.regs[args.rs]
                    .overflowing_add(sign_extend(args.imm, 16))
                    .0;
                self.stats.add_cycles(1);
            }
            Instruction::ADDU(args) => {
                self.regs[args.rd] = self.regs[args.rs].overflowing_add(self.regs[args.rt]).0;
                self.stats.add_cycles(1);
            }
            Instruction::BEQ(args) => {
                if self.regs[args.rs] == self.regs[args.rt] {
                    let target = branch_addr(args.imm);
                    //self.pc = (self.pc as i32 + target) as u32;
                    self.branch_to = Some((self.pc as i32 + target + 4) as u32);
                }
                self.stats.add_cycles(1);
            }
            Instruction::BNE(args) => {
                if self.regs[args.rs] != self.regs[args.rt] {
                    let target = branch_addr(args.imm);
                    self.branch_to = Some((self.pc as i32 + target + 4) as u32);
                }
                self.stats.add_cycles(1);
            }
            Instruction::BLEZ(args) => {
                if as_signed(self.regs[args.rs]) <= 0 {
                    let target = branch_addr(args.imm);
                    self.branch_to = Some((self.pc as i32 + target + 4) as u32);
                }
                self.stats.add_cycles(1);
            }
            Instruction::J(addr) => {
                let target = jump_addr(self.pc, addr);
                self.branch_to = Some(target);
                self.stats.add_cycles(1);
            }
            Instruction::SLT(args) => {
                self.regs[args.rd] =
                    if as_signed(self.regs[args.rs]) < as_signed(self.regs[args.rt]) {
                        1
                    } else {
                        0
                    };
                self.stats.add_cycles(1);
            }
            Instruction::SLTU(args) => {
                self.regs[args.rd] = if self.regs[args.rs] < self.regs[args.rt] {
                    1
                } else {
                    0
                };
                self.stats.add_cycles(1);
            }
            Instruction::JR(args) => {
                self.branch_to = Some(self.regs[args.rs] + 4);
                self.stats.add_cycles(1);
            }
            Instruction::JAL(addr) => {
                self.regs[Register(31)] = self.pc + 4;
                let target = jump_addr(self.pc, addr);
                self.branch_to = Some(target);
                self.stats.add_cycles(1);
            }
            Instruction::SLL(args) => {
                self.regs[args.rd] = self.regs[args.rt] << args.shamt;
                self.stats.add_cycles(1);
            }
            Instruction::SRL(args) => {
                self.regs[args.rd] = self.regs[args.rt] >> args.shamt;
                self.stats.add_cycles(1);
            }
            Instruction::ANDI(args) => {
                self.regs[args.rt] = self.regs[args.rs] & args.imm;
                self.stats.add_cycles(1);
            }
            Instruction::LW(args) => {
                let addr = self.regs[args.rs] as i32 + sign_extend_cast(args.imm, 16);
                let (val, cycles) = self.mem.peek(addr as u32)?;
                self.regs[args.rt] = val;
                self.stats.add_cycles(cycles);
            }
            Instruction::SW(args) => {
                let addr = self.regs[args.rs] as i32 + sign_extend_cast(args.imm, 16);
                let cycles = self.mem.poke(addr as u32, self.regs[args.rt])?;
                self.stats.add_cycles(cycles);
            }
            Instruction::OR(args) => {
                self.regs[args.rd] = self.regs[args.rs] | self.regs[args.rt];
                self.stats.add_cycles(1);
            }
            Instruction::SLTI(args) => {
                self.regs[args.rt] =
                    if as_signed(self.regs[args.rs]) < sign_extend_cast(args.imm, 16) {
                        1
                    } else {
                        0
                    };
                self.stats.add_cycles(1);
            }
            Instruction::JALR(args) => {
                // Note to self:
                // "(...) is the address of the *second* instruction following the branch (...)"
                // por isso o + 4
                self.regs[args.rd] = self.pc + 4;
                self.branch_to = Some(self.regs[args.rs]);
                self.stats.add_cycles(1);
            }
            Instruction::MULT(args) => {
                let a = as_signed(self.regs[args.rs]) as i64;
                let b = as_signed(self.regs[args.rt]) as i64;

                let bytes = (a * b).to_le_bytes();

                let lo = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
                let hi = u32::from_le_bytes(bytes[4..8].try_into().unwrap());

                self.arith_regs = (lo, hi);
                self.stats.add_cycles(1);
            }
            Instruction::MFLO(args) => {
                self.regs[args.rd] = self.arith_regs.0;
                self.stats.add_cycles(1);
            }
            Instruction::MFHI(args) => {
                self.regs[args.rd] = self.arith_regs.1;
                self.stats.add_cycles(1);
            }
            Instruction::DIV(args) => {
                let a = as_signed(self.regs[args.rs]);
                let b = as_signed(self.regs[args.rt]);

                self.arith_regs = (as_unsigned(a / b), as_unsigned(a % b));
                self.stats.add_cycles(1);
            }
            Instruction::LB(args) => {
                let addr = self.regs[args.rs] as i32 + sign_extend_cast(args.imm, 16);
                let (val, cycles) = self.mem.peek(addr as u32)?;
                self.regs[args.rt] = sign_extend(val, 8);
                self.stats.add_cycles(cycles);
            }
            Instruction::LWC1(args) => {
                let addr = self.regs[args.rs] as i32 + sign_extend_cast(args.imm, 16);
                let (val, cycles) = self.mem.peek(addr as u32)?;
                self.float_regs[args.rt.into()] = val;
                self.stats.add_cycles(cycles);
            }
            Instruction::MFC1(args) => {
                self.regs[args.ft.into()] = self.float_regs[args.fs];
                self.stats.add_cycles(1);
            }
            Instruction::LDC1(args) => {
                let addr = self.regs[args.rs] as i32 + sign_extend_cast(args.imm, 16);

                let rt: FloatRegister = args.rt.into();

                let (val_lo, cycles_lo) = self.mem.peek(addr as u32)?;
                let (val_hi, cycles_hi) = self.mem.peek(addr as u32 + 4)?;

                self.float_regs[rt] = val_lo;
                self.float_regs[rt + 1] = val_hi;

                self.stats.add_cycles(cycles_lo);
                self.stats.add_cycles(cycles_hi);
            }
            Instruction::MOV_S(args) => {
                self.float_regs[args.fd] = self.float_regs[args.fs];
                self.stats.add_cycles(1);
            }
            Instruction::ADD_S(args) => {
                let x = word_to_single(self.float_regs[args.fs]);
                let y = word_to_single(self.float_regs[args.ft]);

                let val = x + y;

                self.float_regs[args.fd] = single_to_word(val);
                self.stats.add_cycles(1);
            }
            Instruction::SUB_S(args) => {
                let x = word_to_single(self.float_regs[args.fs]);
                let y = word_to_single(self.float_regs[args.ft]);

                let val = x - y;

                self.float_regs[args.fd] = single_to_word(val);
                self.stats.add_cycles(1);
            }
            Instruction::MUL_S(args) => {
                let x = word_to_single(self.float_regs[args.fs]);
                let y = word_to_single(self.float_regs[args.ft]);

                let val = x * y;

                self.float_regs[args.fd] = single_to_word(val);
                self.stats.add_cycles(1);
            }
            Instruction::DIV_S(args) => {
                let x = word_to_single(self.float_regs[args.fs]);
                let y = word_to_single(self.float_regs[args.ft]);

                let val = x / y;

                self.float_regs[args.fd] = single_to_word(val);
                self.stats.add_cycles(1);
            }
            Instruction::MOV_D(args) => {
                self.float_regs[args.fd] = self.float_regs[args.fs];
                self.float_regs[args.fd + 1] = self.float_regs[args.fs + 1];
                self.stats.add_cycles(1);
            }
            Instruction::ADD_D(args) => {
                let x = dword_to_double(self.float_regs[args.fs], self.float_regs[args.fs + 1]);
                let y = dword_to_double(self.float_regs[args.ft], self.float_regs[args.ft + 1]);

                let (lo, hi) = double_to_dword(x + y);

                self.float_regs[args.fd] = lo;
                self.float_regs[args.fd + 1] = hi;
                self.stats.add_cycles(1);
            }
            Instruction::SUB_D(args) => {
                let x = dword_to_double(self.float_regs[args.fs], self.float_regs[args.fs + 1]);
                let y = dword_to_double(self.float_regs[args.ft], self.float_regs[args.ft + 1]);

                let (lo, hi) = double_to_dword(x - y);

                self.float_regs[args.fd] = lo;
                self.float_regs[args.fd + 1] = hi;
                self.stats.add_cycles(1);
            }
            Instruction::MUL_D(args) => {
                let x = dword_to_double(self.float_regs[args.fs], self.float_regs[args.fs + 1]);
                let y = dword_to_double(self.float_regs[args.ft], self.float_regs[args.ft + 1]);

                let (lo, hi) = double_to_dword(x * y);

                self.float_regs[args.fd] = lo;
                self.float_regs[args.fd + 1] = hi;
                self.stats.add_cycles(1);
            }
            Instruction::DIV_D(args) => {
                let x = dword_to_double(self.float_regs[args.fs], self.float_regs[args.fs + 1]);
                let y = dword_to_double(self.float_regs[args.ft], self.float_regs[args.ft + 1]);

                let (lo, hi) = double_to_dword(x / y);

                self.float_regs[args.fd] = lo;
                self.float_regs[args.fd + 1] = hi;
                self.stats.add_cycles(1);
            }
            Instruction::MTC1(args) => {
                self.float_regs[args.fs] = self.regs[args.ft.into()];
                self.stats.add_cycles(1);
            }
            Instruction::CVT_D_W(args) => {
                let val = as_signed(self.float_regs[args.fs]) as f64;
                let (lo, hi) = double_to_dword(val);
                self.float_regs[args.fd] = lo;
                self.float_regs[args.fd + 1] = hi;
                self.stats.add_cycles(1);
            }
            Instruction::XOR(args) => {
                self.regs[args.rd] = self.regs[args.rs] ^ self.regs[args.rt];
                self.stats.add_cycles(1);
            }
            Instruction::CVT_S_D(args) => {
                let val = dword_to_double(self.float_regs[args.fs], self.float_regs[args.fs + 1]);
                let val = val as f32;
                self.float_regs[args.fd] = single_to_word(val);
                self.stats.add_cycles(1);
            }
            Instruction::AND(args) => {
                self.regs[args.rd] = self.regs[args.rs] & self.regs[args.rt];
                self.stats.add_cycles(1);
            }
            Instruction::SUBU(args) => {
                self.regs[args.rd] = self.regs[args.rs] - self.regs[args.rt];
                self.stats.add_cycles(1);
            }
            Instruction::SRA(args) => {
                let sign = (self.regs[args.rt] & (1 << 31)) >> 31;

                let mut val = self.regs[args.rt] >> args.shamt;
                for i in args.shamt..=31 {
                    val |= sign << i;
                }

                self.regs[args.rd] = val;
                self.stats.add_cycles(1);
            }
            Instruction::BAL(args) => {
                self.regs[Register(31)] = self.pc + 4;
                let target = branch_addr(args.imm);
                self.branch_to = Some((self.pc as i32 + target + 4) as u32);
                self.stats.add_cycles(1);
            }
            Instruction::BGEZ(args) => {
                // Pq a impl de BAL funciona aq, mas BGEZ nao???? lol
                //if as_signed(self.regs[args.rs]) >= 0 {
                //    let target = branch_addr(args.imm);
                //    self.branch_to = Some((self.pc as i32 + target + 4) as u32);
                //}

                if (self.regs[args.rs] & (1 << 31)) == 0 {
                    let target = branch_addr(args.imm);
                    self.branch_to = Some((self.pc as i32 + target + 4) as u32);
                }

                //self.regs[Register(31)] = self.pc + 4;
                //let target = branch_addr(args.imm);
                //self.branch_to = Some((self.pc as i32 + target + 4) as u32);
                self.stats.add_cycles(1);
            }
            Instruction::SWC1(args) => {
                let addr = self.regs[args.rs] as i32 + sign_extend_cast(args.imm, 16);
                let cycles = self
                    .mem
                    .poke(addr as u32, self.float_regs[args.rt.into()])?;
                self.stats.add_cycles(cycles);
            }
            Instruction::C_LT_S(args) => {
                self.float_cc = word_to_single(self.float_regs[args.fs])
                    < word_to_single(self.float_regs[args.ft]);
                //println!("{} < {}? {}", self.float_regs[args.fs], self.float_regs[args.ft], self.float_cc);
                self.stats.add_cycles(1);
            }
            Instruction::BC1T(args) => {
                if self.float_cc {
                    let target = branch_addr(args.imm);
                    self.branch_to = Some((self.pc as i32 + target + 4) as u32);
                }
                self.stats.add_cycles(1);
            }
            Instruction::BC1F(args) => {
                if !self.float_cc {
                    let target = branch_addr(args.imm);
                    self.branch_to = Some((self.pc as i32 + target + 4) as u32);
                }
                self.stats.add_cycles(1);
            }
            a => return Err(eyre!("Instruction {} not implemented yet!", a)),
        }

        // TODO `branch_to.is_some()` é invariante, tirar depois
        //
        // ...pq não posso usar && com `if let`? Deve ter algum RFC pra isso
        if self.in_delay_slot && self.branch_to.is_some() {
            self.pc = self.branch_to.unwrap();
            self.in_delay_slot = false;
            self.branch_to = None;
        } else {
            self.pc += 4;
        }

        Ok(())
    }

    /// Inicia a execução e continua até que ocorra um erro ou a syscall de
    /// parada seja chamada.
    pub fn run(&mut self) -> Result<()> {
        self.stats.start();

        while !self.halt {
            self.cycle()?;
        }

        self.stats.print_stats()?;

        println!();
        println!("Memory Information");
        println!("------------------");
        println!("Level  Hits          Misses        Total          Miss Rate");
        println!("-----  ------------  ------------  ------------   ---------");

        if self.imem.get() as *const u32 != self.mem.get() as *const u32 {
            self.imem.print_stats(false);
        }
        self.mem.print_stats(true);

        Ok(())
    }
}
