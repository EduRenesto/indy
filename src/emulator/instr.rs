use color_eyre::eyre::{Result, eyre};

use super::Register;

pub fn sign_extend(val: u32, init_size: u32) -> u32 {
    let sgn = (val & (1 << (init_size - 1))) >> (init_size - 1);

    if sgn == 0 {
        val
    } else {
        let mut ret = val;

        for i in (init_size)..32 {
            ret |= 1 << i;
        }

        ret
    }
}

pub fn sign_extend_cast(val: u32, init_size: u32) -> i32 {
    i32::from_le_bytes(sign_extend(val, init_size).to_le_bytes())
}

/// (4) BranchAddr no greencard
pub fn branch_addr(val: u32) -> i32 {
    let fifteenth_bit = (val & (1 << 15)) >> 15;
    let mut val = 0 | (val << 2);
    for i in 17..=31 {
        val |= fifteenth_bit << i;
    }
    i32::from_le_bytes(val.to_le_bytes())
}

/// (5) JumpAddr no greencard
pub fn jump_addr(pc: u32, val: u32) -> u32 {
    let high_pc = (pc + 4) & (0xF0000000);
    (high_pc) | (val << 2)
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
    BEQ(IArgs),
    J(u32),
    BNE(IArgs),
    SLT(RArgs),
    JR(RArgs),
    JAL(u32),
    SLL(RArgs),
    SRL(RArgs),
    ANDI(IArgs),
    LW(IArgs),
    SW(IArgs),
}

impl std::fmt::Display for Instruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            &Instruction::SYSCALL => write!(f, "SYSCALL"),
            &Instruction::ADD(ref a) => write!(f, "ADD {}, {}, {}", a.rd, a.rs, a.rt),
            &Instruction::ADDI(ref a) => write!(f, "ADDI {}, {}, {}", a.rt, a.rs, sign_extend_cast(a.imm, 16)),
            &Instruction::LUI(ref a) => write!(f, "LUI {}, {}", a.rt, a.imm),
            &Instruction::ORI(ref a) => write!(f, "ORI {}, {}, {}", a.rt, a.rs, a.imm),
            &Instruction::ADDIU(ref a) => write!(f, "ADDIU {}, {}, {}", a.rt, a.rs, a.imm),
            &Instruction::ADDU(ref a) => write!(f, "ADDU {}, {}, {}", a.rd, a.rs, a.rt),
            &Instruction::BEQ(ref a) => write!(f, "BEQ {}, {}, {}", a.rs, a.rt, a.imm),
            &Instruction::J(ref a) => write!(f, "J {:#x} # {:#x}", a, a * 4),
            &Instruction::BNE(ref a) => write!(f, "BNE {}, {}, {}", a.rs, a.rt, a.imm),
            &Instruction::SLT(ref a) => write!(f, "SLT {}, {}, {}", a.rd, a.rs, a.rt),
            &Instruction::JR(ref a) => write!(f, "JR {}", a.rs),
            &Instruction::JAL(ref a) => write!(f, "JAL {:#x} # {:#x}", a, a * 4),
            &Instruction::SLL(ref a) => write!(f, "SLL {}, {}, {}", a.rd, a.rs, a.rt),
            &Instruction::SRL(ref a) => write!(f, "SRL {}, {}, {}", a.rd, a.rs, a.rt),
            &Instruction::ANDI(ref a) => write!(f, "ANDI {}, {}, {}", a.rt, a.rs, a.imm),
            &Instruction::LW(ref a) => write!(f, "LW {}, {}, {}", a.rt, a.rs, a.imm),
            &Instruction::SW(ref a) => write!(f, "SW {}, {}, {}", a.rt, a.rs, a.imm),
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
        0x2A => Ok(Instruction::SLT(RArgs { rd, rt, rs, shamt })),
        0x08 => Ok(Instruction::JR(RArgs { rd, rt, rs, shamt })),
        0x00 => Ok(Instruction::SLL(RArgs { rd, rt, rs, shamt })),
        0x02 => Ok(Instruction::SRL(RArgs { rd, rt, rs, shamt })),
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
        0x04 => Ok(Instruction::BEQ(IArgs { rs, rt, imm })),
        0x05 => Ok(Instruction::BNE(IArgs { rs, rt, imm })),
        0x0C => Ok(Instruction::ANDI(IArgs { rs, rt, imm })),
        0x23 => Ok(Instruction::LW(IArgs { rs, rt, imm })),
        0x2B => Ok(Instruction::SW(IArgs { rs, rt, imm })),
        _ => Err(eyre!("Unknown I instruction: {:#x}", opcode)),
    }
}
fn decode_j_instr(word: u32) -> Result<Instruction> {
    let opcode = (word & (63 << 26)) >> 26;

    // 26 least significant bytes
    let target = word & 0x3FFFFFF;
    match opcode {
        0x02 => Ok(Instruction::J(target)),
        0x03 => Ok(Instruction::JAL(target)),
        _ => Err(eyre!("Unknown J instruction: {:#x}", opcode))
    }
}
