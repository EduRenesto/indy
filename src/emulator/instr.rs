use color_eyre::eyre::{Result, eyre};

use super::Register;

pub fn sign_extend(val: u32, init_size: u32) -> u32 {
    let sgn = (val & (1 << (init_size - 1))) >> (init_size - 1);

    if sgn == 0 {
        val
    } else {
        let mut ret = val;

        for i in (init_size + 1)..32 {
            ret |= 1 << i;
        }

        ret
    }
}

pub struct RArgs {
    pub rs: Register,
    pub rt: Register,
    pub rd: Register,
    pub shamt: u32,
}

pub struct IArgs {
    pub rs: Register,
    pub rt: Register,
    pub imm: u32,
}

/// TODO melhorar docs
#[allow(snake_case)]
pub enum Instruction {
    ADD(RArgs),
    ADDI(IArgs),
    SYSCALL,
    LUI(IArgs),
    ORI(IArgs),
    ADDIU(IArgs),
    ADDU(RArgs),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Instruction::SYSCALL => write!(f, "SYSCALL"),
            &Instruction::ADD(ref a) => write!(f, "ADD {}, {}, {}", a.rd, a.rs, a.rt),
            &Instruction::ADDI(ref a) => write!(f, "ADDI {}, {}, {}", a.rt, a.rs, a.imm),
            &Instruction::LUI(ref a) => write!(f, "LUI {}, {}", a.rt, a.imm),
            &Instruction::ORI(ref a) => write!(f, "ORI {}, {}, {}", a.rt, a.rs, a.imm),
            &Instruction::ADDIU(ref a) => write!(f, "ADDIU {}, {}, {}", a.rt, a.rs, a.imm),
            &Instruction::ADDU(ref a) => write!(f, "ADDU {}, {}, {}", a.rd, a.rs, a.rt),
        }
    }
}

impl Instruction {
    pub fn decode(word: u32) -> Result<Instruction> {
        let opcode = (word & (63 << 26)) >> 26;

        //println!("word: {:#x}, opcode: {:#x}", word, opcode);

        match opcode {
            0 => decode_r_instr(word),
            2 | 3 => decode_j_instr(word),
            _ => decode_i_instr(word),
        }
    }
}

fn decode_r_instr(word: u32) -> Result<Instruction> {
    // Considerando unsigned.
    // TODO fazer sign extension
    let funct = word & 63;
    let shamt = (word & (31 << 6)) >> 6;
    let rd = Register((word & (31 << 11)) >> 11);
    let rt = Register((word & (31 << 16)) >> 16);
    let rs = Register((word & (31 << 21)) >> 21);

    match funct {
        0x0C => Ok(Instruction::SYSCALL),
        0x20 => Ok(Instruction::ADD(RArgs { rd, rt, rs, shamt })),
        0x21 => Ok(Instruction::ADDU(RArgs { rd, rt, rs, shamt })),
        _ => Err(eyre!("Unknown R instruction: {:#x}", funct)),
    }
}
fn decode_i_instr(word: u32) -> Result<Instruction> {
    // Considerando unsigned.
    // TODO fazer sign extension
    let imm = word & 0xFFFF;
    let rt = Register((word & (31 << 16)) >> 16);
    let rs = Register((word & (31 << 21)) >> 21);
    let opcode = (word & (63 << 26)) >> 26;

    match opcode {
        0x08 => Ok(Instruction::ADDI(IArgs { rs, rt, imm })),
        0x0F => Ok(Instruction::LUI(IArgs { rs, rt, imm, })),
        0x0D => Ok(Instruction::ORI(IArgs { rs, rt, imm, })),
        0x09 => Ok(Instruction::ADDIU(IArgs { rs, rt, imm })),
        _ => Err(eyre!("Unknown I instruction: {:#x}", opcode)),
    }
}
fn decode_j_instr(word: u32) -> Result<Instruction> {
    Err(eyre!("J not implemented: {:#x}", word))
}
