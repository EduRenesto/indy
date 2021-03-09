use proc_macro::TokenStream;
use quote::quote;

use std::fs::File;

mod decl;
mod instruction;
mod fmt;
mod parse;

use instruction::Instructions;

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
        use crate::emulator::Register;
        use crate::emulator::instr::sign_extend_cast;

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

        #decl

        #fmt

        #parse
    };

    code.into()
}
