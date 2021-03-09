use crate::instruction::*;

use quote::quote;
use proc_macro2::{ Span, Ident, TokenStream };

/// Gera o *match pattern* e o pretty-print/disassembly de uma instrução do
/// tipo R.
fn generate_r_fmt((name, instr): (&String, &RInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let fmt: TokenStream = if instr.shift.unwrap_or(false) {
        let c = quote! {
            write!(f, "{} {}, {}, {}", #ename, a.rd, a.rt, a.shamt)
        };

        c.into()
    } else if instr.one_operand.unwrap_or(false) {
        let c = quote! {
            write!(f, "{} {}", #ename, a.rs)
        };

        c.into()
    } else if instr.has_args.unwrap_or(true) {
        let c = quote! {
            write!(f, "{} {}, {}, {}", #ename, a.rd, a.rs, a.rt)
        };
        
        c.into()
    } else {
        let c = quote! {
            write!(f, "{}", #ename)
        };

        c.into()
    };

    let code = quote! {
        &Instruction:: #ename_ident (ref a) => #fmt,
    };

    code.into()
}


/// Gera o *match pattern* e o pretty-print/disassembly de uma instrução do
/// tipo I.
fn generate_i_fmt((name, instr): (&String, &IInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let fmt: TokenStream = if instr.load_store.unwrap_or(false) {
        let c = quote! {
            write!(f, "{} {}, {:#x}({})", #ename, a.rt, sign_extend_cast(a.imm, 16), a.rs)
        };
        
        c.into()
    } else if instr.half_word.unwrap_or(false) {
        let c = quote! {
            write!(f, "{} {}, {}", #ename, a.rt, a.imm)
        };
        
        c.into()
    } else if instr.invert.unwrap_or(false) {
        let c = quote! {
            write!(f, "{} {}, {}, {}", #ename, a.rs, a.rt, sign_extend_cast(a.imm, 16))
        };
        
        c.into()
    } else if instr.sign_ext.unwrap_or(true) {
        let c = quote! {
            write!(f, "{} {}, {}, {}", #ename, a.rt, a.rs, sign_extend_cast(a.imm, 16))
        };
        
        c.into()
    } else {
        let c = quote! {
            write!(f, "{} {}, {}, {}", #ename, a.rt, a.rs, a.imm)
        };

        c.into()
    };

    let code = quote! {
        &Instruction:: #ename_ident (ref a) => #fmt,
    };

    code.into()
}

/// Gera o *match pattern* e o pretty-print/disassembly de uma instrução do
/// tipo J.
fn generate_j_fmt((name, _instr): (&String, &JInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        &Instruction:: #ename_ident (ref a) => write!(f, "{} {:#x} # {:#x}", #ename, a, a * 4),
    };

    code.into()
}

/// Gera a implementação de `std::fmt::Display` para a enum `Instruction`. Ou
/// seja, gera o pretty-printing/disassembly para as instruções.
pub(crate) fn generate_fmt(instrs: &Instructions) -> TokenStream {
    let r = instrs.r
        .iter()
        .map(generate_r_fmt)
        .collect::<Vec<_>>();
    let i = instrs.i
        .iter()
        .map(generate_i_fmt)
        .collect::<Vec<_>>();
    let j = instrs.j
        .iter()
        .map(generate_j_fmt)
        .collect::<Vec<_>>();

    let code = quote! {
        impl std::fmt::Display for Instruction {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                match self {
                    #(#r)
                    *
                    #(#i)
                    *
                    #(#j)
                    *
                }
            }
        }
    }.into();

    code
}
