//! Algumas operações úteis para trabalhar com instruções. Antes do grande
//! overhaul com a macro, o decode das instruções ficava aqui.

/// Faz extensão de sinal em `val`, supondo que o tamanho anterior do inteiro
/// era `init_size` bits.
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

/// Faz extensão de sinal em `val`, supondo que o tamanho anterior do inteiro
/// era `init_size` bits, e reinterpreta como um signed de 32 bits.
pub fn sign_extend_cast(val: u32, init_size: u32) -> i32 {
    i32::from_le_bytes(sign_extend(val, init_size).to_le_bytes())
}

/// Calcula (4) BranchAddr no greencard
pub fn branch_addr(val: u32) -> i32 {
    let fifteenth_bit = (val & (1 << 15)) >> 15;
    let mut val = 0 | (val << 2);
    for i in 17..=31 {
        val |= fifteenth_bit << i;
    }
    i32::from_le_bytes(val.to_le_bytes())
}

/// Calcula (5) JumpAddr no greencard
pub fn jump_addr(pc: u32, val: u32) -> u32 {
    let high_pc = (pc + 4) & (0xF0000000);
    (high_pc) | (val << 2)
}
