use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use quote::ToTokens;

use serde::Deserialize;

use std::fs::File;
use std::io::Read;

#[derive(Deserialize)]
struct Instructions {
    r: HashMap<String, u32>,
    i: HashMap<String, u32>,
    j: HashMap<String, u32>,
}

fn to_r_instruction(instr: (&String, &u32)) -> proc_macro2::TokenStream {
    let name = proc_macro2::Ident::new(&instr.0.to_uppercase(), proc_macro2::Span::call_site());

    let code = quote! {
        #name (RArgs)
    };

    code.into()
}

fn to_i_instruction(instr: (&String, &u32)) -> proc_macro2::TokenStream {
    let name = proc_macro2::Ident::new(&instr.0.to_uppercase(), proc_macro2::Span::call_site());

    let code = quote! {
        #name (IArgs)
    };

    code.into()
}

fn to_j_instruction(instr: (&String, &u32)) -> proc_macro2::TokenStream {
    let name = proc_macro2::Ident::new(&instr.0.to_uppercase(), proc_macro2::Span::call_site());

    let code = quote! {
        #name (u32)
    };

    code.into()
}

fn gen_opcode_case(instr: (&String, &u32)) -> proc_macro2::TokenStream {
    let opcode = instr.1;
    let mnemonic = proc_macro2::Ident::new(&instr.0.to_uppercase(), proc_macro2::Span::call_site());
    let code = quote! {
        #opcode => Ok(Instruction::#mnemonic (args))
    };
    code.into()
}

fn gen_parse_r_instr(instr: &HashMap<String, u32>) -> proc_macro2::TokenStream {
    let cases = instr.iter().map(gen_opcode_case).collect::<Vec<_>>();

    let code = quote! {
        fn decode_r_instr(word: u32) -> Result<Instruction> {
            let funct = word & 63;
            let shamt = (word & (31 << 6)) >> 6;
            let rd = Register((word & (31 << 11)) >> 11);
            let rt = Register((word & (31 << 16)) >> 16);
            let rs = Register((word & (31 << 21)) >> 21);

            let args = RArgs { rd, rt, rs, shamt };

            match funct {
                #(#cases),
                *,
                _ => Err(eyre!("Unknown R instruction {:#010x}", funct)),
            }
        }
    };

    code.into()
}

fn gen_parse_i_instr(instr: &HashMap<String, u32>) -> proc_macro2::TokenStream {
    let cases = instr.iter().map(gen_opcode_case).collect::<Vec<_>>();
    let code = quote! {
        fn decode_i_instr(word: u32) -> Result<Instruction> {
            // Considerando unsigned.
            // TODO fazer sign extension
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
    };

    code.into()
}

fn gen_parse_j_instr(instr: &HashMap<String, u32>) -> proc_macro2::TokenStream {
    let cases = instr.iter().map(gen_opcode_case).collect::<Vec<_>>();
    let code = quote! {
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
    };

    code.into()
}

fn gen_fmt_r((instr, _): (&String, &u32)) -> proc_macro2::TokenStream {
    let opcode = instr.to_uppercase();
    let code = quote! {
        Instruction::#opcode (ref a) => write!(f, "#opcode {} {} {}", a.rd, a.rs, a.rt),
    };

    code.into()
}
fn gen_fmt_i((instr, _): (&String, &u32)) -> proc_macro2::TokenStream {
    let opcode = instr.to_uppercase();
    let code = quote! {
        Instruction::#opcode (ref a) => write!(f, "#opcode {} {} {}", a.rd, a.rs, a.rt),
    };

    code.into()
}
fn gen_fmt_j((instr, _): (&String, &u32)) -> proc_macro2::TokenStream {
    let opcode = instr.to_uppercase();
    let code = quote! {
        Instruction::#opcode (ref a) => write!(f, "#opcode {} {} {}", a.rd, a.rs, a.rt),
    };

    code.into()
}

#[proc_macro]
pub fn instr_from_yaml(item: TokenStream) -> TokenStream {
    let file = item.to_string();
    let file = format!("{}/../{}", env!("CARGO_MANIFEST_DIR"), &file[1..file.len()-1]);

    let file = File::open(file).unwrap();
    let instructions: Instructions = serde_yaml::from_reader(file).expect("Failed to parse instructions file");

    //let r = instructions.r
    //    .iter()
    //    .map(|(k, v)| {
    //        let mnemonic = k.to_uppercase();
    //        let c = quote! {
    //            #mnemonic
    //        };
    //        c
    //    }).collect::<Vec<_>>();

    let r_instr = instructions.r.iter()
        .map(to_r_instruction)
        .collect::<Vec<_>>();
    let i_instr = instructions.i.iter()
        .map(to_i_instruction)
        .collect::<Vec<_>>();
    let j_instr = instructions.j.iter()
        .map(to_j_instruction)
        .collect::<Vec<_>>();

    let r = quote! {
        #(#r_instr),
        *
    };
    let i = quote! {
        #(#i_instr),
        *
    };
    let j = quote! {
        #(#j_instr),
        *
    };

    let parse_r = gen_parse_r_instr(&instructions.r);
    let parse_i = gen_parse_i_instr(&instructions.i);
    let parse_j = gen_parse_j_instr(&instructions.j);

    let code = quote! {
        use color_eyre::eyre::{ eyre, Result };
        use crate::emulator::Register;

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

        pub enum Instruction {
            #r,
            #i,
            #j
        }

        #parse_r
        #parse_i
        #parse_j

        impl Instruction {
            pub fn decode(word: u32) -> Result<Instruction> {
                let opcode = (word & (63 << 26)) >> 26;

                match opcode {
                    0 => decode_r_instr(word),
                    2 | 3 => decode_j_instr(word),
                    _ => decode_i_instr(word),
                }
            }
        }
    };

    code.into()
}
