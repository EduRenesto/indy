use crate::instruction::*;

use quote::quote;
use proc_macro2::{ Span, Ident, TokenStream };

fn generate_r_fmt((name, instr): (&String, &RInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let fmt: TokenStream = if instr.has_args.unwrap_or(true) {
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

fn generate_i_fmt((name, instr): (&String, &IInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let fmt: TokenStream = if instr.sign_ext.unwrap_or(true) {
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

fn generate_j_fmt((name, _instr): (&String, &JInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        &Instruction:: #ename_ident (ref a) => write!(f, "{} {:#010} # {:#010}", #ename, a, a * 4),
    };

    code.into()
}

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
