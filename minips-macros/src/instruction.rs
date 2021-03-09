use std::collections::HashMap;

use serde::Deserialize;

#[derive(Deserialize)]
pub(crate) struct RInstruction {
    pub(crate) opcode: Option<u32>,
    pub(crate) funct: u32,
    pub(crate) has_args: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct IInstruction {
    pub(crate) opcode: u32,
    pub(crate) sign_ext: Option<bool>,
}

#[derive(Deserialize)]
pub(crate) struct JInstruction {
    pub(crate) opcode: u32,
}

#[derive(Deserialize)]
pub(crate) struct Instructions {
    pub r: HashMap<String, RInstruction>,
    pub i: HashMap<String, IInstruction>,
    pub j: HashMap<String, JInstruction>,
}
