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

/// Gera um `match pattern` para uma instrução do tipo FR.
fn generate_fr_parse_case((name, instr): (&String, &FRInstruction)) -> TokenStream {
    let ename = name.to_uppercase().replace(".", "_");
    let ename_ident = Ident::new(&ename, Span::call_site());

    let opcode = instr.opcode;
    let fmt = instr.fmt;
    let funct = instr.funct;

    let code = quote! {
        (#opcode, #fmt) if funct == #funct => Ok(Instruction::#ename_ident (rargs))
    };

    code.into()
}

/// Gera um `match pattern` para uma instrução do tipo FI.
fn generate_fi_parse_case((name, instr): (&String, &FIInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let opcode = instr.opcode;
    let fmt = instr.fmt;
    let ft = instr.ft;

    let code = quote! {
        (#opcode, #fmt) if ft == #ft => Ok(Instruction::#ename_ident (iargs))
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

/// Gera a função que faz o parsing de uma instrução de ponto flutuante.
fn generate_f_parse(fr: &HashMap<String, FRInstruction>, fi: &HashMap<String, FIInstruction>) -> TokenStream {
    let fr_cases = fr
        .iter()
        .map(generate_fr_parse_case)
        .collect::<Vec<_>>();
    let fi_cases = fi
        .iter()
        .map(generate_fi_parse_case)
        .collect::<Vec<_>>();

    let code = quote!{
        /// Faz o parse de uma instrução do tipo I.
        fn decode_f_instr(word: u32) -> Result<Instruction> {
            // for both FR and FI
            let ft = ((word & (31 << 16)) >> 16);
            let fmt = ((word & (31 << 21)) >> 21);
            let opcode = (word & (63 << 26)) >> 26;

            // for FR
            let funct = word & 63;
            let fd = (word & (31 << 6)) >> 6;
            let fs = ((word & (31 << 11)) >> 11);

            // for FI
            let imm = word & 0xFFFF;

            let rargs = FRArgs { ft: FloatRegister(ft), fs: FloatRegister(fs), fd: FloatRegister(fd), funct };
            let iargs = FIArgs { ft: FloatRegister(ft), imm };

            //println!("F: (opcode, fmt, funct) = ({:#x},{:#x},{:#x})", opcode, fmt, funct);

            match (opcode, fmt) {
                #(#fr_cases),
                *,
                #(#fi_cases),
                *,
                _ => Err(eyre!("Unknown F instruction: {:#x}/{:#x}", opcode, fmt)),
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

    let parse_f = generate_f_parse(&instrs.fr, &instrs.fi);

    let code = quote! {
        #parse_r

        #parse_i

        #parse_j

        #parse_f

        impl Instruction {
            /// Recebe uma palavra de 32 bits e tenta decodificá-la em uma
            /// instrução.
            pub fn decode(word: u32) -> Result<Instruction> {
                if word == 0 {
                    return Ok(Instruction::NOP);
                }

                let opcode = (word & (63 << 26)) >> 26;

                match opcode {
                    0 | 16 => decode_r_instr(word),
                    2 | 3 => decode_j_instr(word),
                    17 => decode_f_instr(word),
                    _ => decode_i_instr(word),
                }
            }
        }
    }.into();

    code
}
