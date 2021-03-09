use std::collections::HashMap;

use crate::instruction::*;

use quote::quote;
use proc_macro2::{ Span, Ident, TokenStream };

/// Gera um `match pattern` para uma instrução do tipo R.
fn generate_r_parse_case((name, instr): (&String, &RInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());
    let opcode = instr.opcode.as_ref().unwrap_or(&0x0);
    let funct = instr.funct;

    let code = quote! {
        (#opcode, #funct) => Ok(Instruction::#ename_ident (args))
    };

    code.into()
}

/// Gera um `match pattern` para uma instrução do tipo I.
fn generate_i_parse_case((name, instr): (&String, &IInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());
    let opcode = instr.opcode;

    let code = quote! {
        #opcode => Ok(Instruction::#ename_ident (args))
    };

    code.into()
}

/// Gera um `match pattern` para uma instrução do tipo J.
fn generate_j_parse_case((name, instr): (&String, &JInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());
    let opcode = instr.opcode;

    let code = quote! {
        #opcode => Ok(Instruction::#ename_ident (args))
    };

    code.into()
}

/// Gera a função que faz o parsing de uma instrução do tipo R.
fn generate_r_parse(instrs: &HashMap<String, RInstruction>) -> TokenStream {
    let cases = instrs
        .iter()
        .map(generate_r_parse_case)
        .collect::<Vec<_>>();

    let code = quote!{
        /// Faz o parse de uma instrução do tipo R.
        fn decode_r_instr(word: u32) -> Result<Instruction> {
            let funct = word & 63;
            let shamt = (word & (31 << 6)) >> 6;
            let rd = Register((word & (31 << 11)) >> 11);
            let rt = Register((word & (31 << 16)) >> 16);
            let rs = Register((word & (31 << 21)) >> 21);
            let opcode = (word & (63 << 26)) >> 26;

            let args = RArgs { rd, rt, rs, shamt };

            match (opcode, funct) {
                #(#cases),
                *,
                _ => Err(eyre!("Unknown R instruction {:#010x}/{:#010x}", opcode, funct)),
            }
        }
    }.into();

    code
}

/// Gera a função que faz o parsing de uma instrução do tipo I.
fn generate_i_parse(instrs: &HashMap<String, IInstruction>) -> TokenStream {
    let cases = instrs
        .iter()
        .map(generate_i_parse_case)
        .collect::<Vec<_>>();

    let code = quote!{
        /// Faz o parse de uma instrução do tipo I.
        fn decode_i_instr(word: u32) -> Result<Instruction> {
            let imm = word & 0xFFFF;
            let rt = Register((word & (31 << 16)) >> 16);
            let rs = Register((word & (31 << 21)) >> 21);
            let opcode = (word & (63 << 26)) >> 26;

            let args = IArgs { rs, rt, imm };

            match opcode {
                #(#cases),
                *,
                _ => Err(eyre!("Unknown I instruction: {:#x}", opcode)),
            }
        }
    }.into();

    code
}

/// Gera a função que faz o parsing de uma instrução do tipo J.
fn generate_j_parse(instrs: &HashMap<String, JInstruction>) -> TokenStream {
    let cases = instrs
        .iter()
        .map(generate_j_parse_case)
        .collect::<Vec<_>>();

    let code = quote!{
        /// Faz o parse de uma instrução do tipo J.
        fn decode_j_instr(word: u32) -> Result<Instruction> {
            let opcode = (word & (63 << 26)) >> 26;

            // 26 least significant bytes
            let args = word & 0x3FFFFFF;
            match opcode {
                #(#cases),
                *,
                _ => Err(eyre!("Unknown J instruction: {:#x}", opcode)),
            }
        }
    }.into();

    code
}

/// Gera a função que transforma uma palavra de 32 bits em uma variante da
/// enum `Instruction`.
pub(crate) fn generate_parse(instrs: &Instructions) -> TokenStream {
    let parse_r = generate_r_parse(&instrs.r);
    let parse_i = generate_i_parse(&instrs.i);
    let parse_j = generate_j_parse(&instrs.j);

    let code = quote! {
        #parse_r

        #parse_i

        #parse_j

        impl Instruction {
            /// Recebe uma palavra de 32 bits e tenta decodificá-la em uma
            /// instrução.
            pub fn decode(word: u32) -> Result<Instruction> {
                let opcode = (word & (63 << 26)) >> 26;

                match opcode {
                    0 | 16 => decode_r_instr(word),
                    2 | 3 => decode_j_instr(word),
                    _ => decode_i_instr(word),
                }
            }
        }
    }.into();

    code
}
