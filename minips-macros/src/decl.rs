use crate::instruction::*;

use quote::quote;
use proc_macro2::{ Span, Ident, TokenStream };

fn generate_r_decl((name, _instr): (&String, &RInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        #ename_ident (RArgs),
    };

    code.into()
}

fn generate_i_decl((name, _instr): (&String, &IInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        #ename_ident (IArgs),
    };

    code.into()
}

fn generate_j_decl((name, _instr): (&String, &JInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        #ename_ident (u32),
    };

    code.into()
}

pub(crate) fn generate_decl(instrs: &Instructions) -> TokenStream {
    let r = instrs.r
        .iter()
        .map(generate_r_decl)
        .collect::<Vec<_>>();
    let i = instrs.i
        .iter()
        .map(generate_i_decl)
        .collect::<Vec<_>>();
    let j = instrs.j
        .iter()
        .map(generate_j_decl)
        .collect::<Vec<_>>();

    let code = quote! {
        pub enum Instruction {
            #(#r)
            *
            #(#i)
            *
            #(#j)
            *
        }
    }.into();

    code
}
