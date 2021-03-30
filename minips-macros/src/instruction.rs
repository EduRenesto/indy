//! Esse módulo contém as estruturas que são lidas do arquivo YAML. As flags
//! aqui definidas podem ser utilizadas nas instruções lá.

use std::collections::HashMap;

use serde::Deserialize;

/// Uma instrução do tipo R.
///
/// TODO: remover `has_args`, `one_operand`, `two_operads` e colocar
/// um `n_operands: Option<u32>`.
#[derive(Deserialize)]
pub(crate) struct RInstruction {
    pub(crate) opcode: Option<u32>,
    pub(crate) funct: u32,
    pub(crate) has_args: Option<bool>,
    pub(crate) shift: Option<bool>,
    pub(crate) one_operand: Option<bool>,
    pub(crate) two_operands: Option<bool>,
    pub(crate) move_cop: Option<bool>,
}

/// Uma instrução do tipo I.
#[derive(Deserialize)]
pub(crate) struct IInstruction {
    pub(crate) opcode: u32,
    pub(crate) sign_ext: Option<bool>,
    pub(crate) load_store: Option<bool>,
    pub(crate) half_word: Option<bool>,
    pub(crate) invert: Option<bool>,
    pub(crate) target_is_float: Option<bool>,
}

/// Uma instrução do tipo J.
#[derive(Deserialize)]
pub(crate) struct JInstruction {
    pub(crate) opcode: u32,
}

/// Uma instrução do tipo FR.
#[derive(Deserialize)]
pub(crate) struct FRInstruction {
    pub(crate) opcode: u32,
    pub(crate) fmt: u32,
    pub(crate) funct: u32,
    pub(crate) two_operands: Option<bool>,
    pub(crate) first_is_float: Option<bool>,
}

/// Uma instrução do tipo FI.
#[derive(Deserialize)]
pub(crate) struct FIInstruction {
    pub(crate) opcode: u32,
    pub(crate) fmt: u32,
    pub(crate) ft: u32,
}

/// O conjunto de todas as instruções do arquivo YAML.
#[derive(Deserialize)]
pub(crate) struct Instructions {
    pub r: HashMap<String, RInstruction>,
    pub i: HashMap<String, IInstruction>,
    pub j: HashMap<String, JInstruction>,

    pub fr: HashMap<String, FRInstruction>,
    pub fi: HashMap<String, FIInstruction>,
}
