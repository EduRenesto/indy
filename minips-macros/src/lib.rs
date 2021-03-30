//! minips-macros: implementação das proc-macros do projeto minips-rs
//! 
//! Essa subcrate implementa a procedural macro `instr_from_yaml`, responsável
//! por parsear o arquivo `instructions.yml` e gerar a declaração das
//! instruções, assim como seu pretty-printing e decoding.
//!
//! Cada submódulo implementa uma parte da macro:
//! - decl: as declarações das instruções
//! - instruction: parsing do arquivo yaml
//! - fmt: pretty-printing/disassembly
//! - parse: decoding

use proc_macro::TokenStream;
use quote::quote;

use std::fs::File;

mod decl;
mod instruction;
mod fmt;
mod parse;

use instruction::Instructions;

/// Gera a declaração e implementação das instruções descritas por um arquivo
/// YAML.
///
/// Uso:
/// ```rust
/// instr_from_yaml!(instructions.yaml)
/// ```
#[proc_macro]
pub fn instr_from_yaml(item: TokenStream) -> TokenStream {
    let file = item.to_string();
    let file = format!("{}/../{}", env!("CARGO_MANIFEST_DIR"), &file[1..file.len()-1]);

    let file = File::open(file).expect("Failed to open instruction file");
    let instructions: Instructions = serde_yaml::from_reader(file).expect("Failed to parse instructions file");

    let decl = decl::generate_decl(&instructions);
    let fmt = fmt::generate_fmt(&instructions);
    let parse = parse::generate_parse(&instructions);

    let code = quote! {
        use color_eyre::eyre::{ eyre, Result };
        use crate::emulator::{ Register, FloatRegister };
        use crate::emulator::instr::sign_extend_cast;

        /// Operandos contidos numa intrução do tipo R.
        pub struct RArgs {
            pub rs: Register,
            pub rt: Register,
            pub rd: Register,
            pub shamt: u32,
        }

        /// Operandos contidos numa intrução do tipo I.
        pub struct IArgs {
            pub rs: Register,
            pub rt: Register,
            pub imm: u32,
        }

        /// Operandos contidos numa instrução do tipo FR.
        pub struct FRArgs {
            pub ft: FloatRegister,
            pub fs: FloatRegister,
            pub fd: FloatRegister,
            pub funct: u32,
        }

        /// Operandos contidos numa instrução do tipo FI.
        pub struct FIArgs {
            pub ft: FloatRegister,
            pub imm: u32,
        }

        #decl

        #fmt

        #parse
    };

    code.into()
}
