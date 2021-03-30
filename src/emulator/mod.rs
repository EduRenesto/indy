//! Esse módulo encapsula o core do emulador.

pub(crate) mod cpu;
pub(crate) mod instr;
pub(crate) mod memory;

// Re-exports pra ficar melhor de usar ao longo do código
pub use cpu::Cpu;
//pub use instr::{IArgs, Instruction, RArgs};
pub use memory::Memory;

/// Mais uma vez usamos o `newtype`. Essa struct não é um registrador em si
/// (vide `cpu.rs`), mas um *índice* para um registrador.
///
/// Esse idiom é útil aqui para podermos implementar os aliases dos
/// registradores para o disassemble.
#[derive(Copy, Clone)]
pub struct Register(u32);

impl std::fmt::Display for Register {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self.0 {
            0 => write!(f, "$zero"),
            1 => write!(f, "$at"),
            2..=3 => write!(f, "$v{}", self.0 - 2),
            4..=7 => write!(f, "$a{}", self.0 - 4),
            8..=15 => write!(f, "$t{}", self.0 - 8),
            16..=23 => write!(f, "$s{}", self.0 - 16),
            24..=25 => write!(f, "$t{}", self.0 - 24 + 8),
            28 => write!(f, "$gp"),
            29 => write!(f, "$sp"),
            30 => write!(f, "$fp"),
            31 => write!(f, "$ra"),
            _ => write!(f, "$!!!"),
        }
    }
}

/// Um índice para um registrador de ponto flutuante.
pub struct FloatRegister(u32);

impl std::convert::From<Register> for FloatRegister {
    fn from(reg: Register) -> Self {
        FloatRegister(reg.0)
    }
}

impl std::fmt::Display for FloatRegister {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "$f{}", self.0)
    }
}

/// Esse módulo contém a declaração das instruções e o decoder das mesmas.
/// Eles são gerados automaticamente pela macro `instr_from_yaml` usando o
/// arquivo `instructions.yml`, que está na raíz do projeto. A implementação
/// dessa macro está na subcrate `minips-macros`.
mod autogen {
    use minips_macros::instr_from_yaml;

    instr_from_yaml!("instructions.yml");
}

pub use autogen::*;
