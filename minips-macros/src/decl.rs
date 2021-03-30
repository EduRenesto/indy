use crate::instruction::*;

use quote::quote;
use proc_macro2::{ Span, Ident, TokenStream };

/// Gera o *enum item* para uma instrução do tipo R.
fn generate_r_decl((name, _instr): (&String, &RInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        #ename_ident (RArgs),
    };

    code.into()
}

/// Gera o *enum item* para uma instrução do tipo I.
fn generate_i_decl((name, _instr): (&String, &IInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        #ename_ident (IArgs),
    };

    code.into()
}

/// Gera o *enum item* para uma instrução do tipo J.
fn generate_j_decl((name, _instr): (&String, &JInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        #ename_ident (u32),
    };

    code.into()
}

/// Gera o *enum item* para uma instrução do tipo FR.
fn generate_fr_decl((name, _instr): (&String, &FRInstruction)) -> TokenStream {
    let ename = name.to_uppercase().replace(".", "_");
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        #ename_ident (FRArgs),
    };

    code.into()
}

/// Gera o *enum item* para uma instrução do tipo FR.
fn generate_fi_decl((name, _instr): (&String, &FIInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        #ename_ident (FIArgs),
    };

    code.into()
}

/// Gera um enum declarando as instruções.
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
    let fr = instrs.fr
        .iter()
        .map(generate_fr_decl)
        .collect::<Vec<_>>();
    let fi = instrs.fi
        .iter()
        .map(generate_fi_decl)
        .collect::<Vec<_>>();

    let code = quote! {
        /// As instruções MIPS, geradas a partir da macro `instr_from_yaml`.
        pub enum Instruction {
            NOP,

            #(#r)
            *
            #(#i)
            *
            #(#j)
            *
            #(#fr)
            *
            #(#fi)
            *

            //MFC1(Register, FloatRegister),
        }
    }.into();

    code
}
