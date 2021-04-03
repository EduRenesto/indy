use crate::instruction::*;

use quote::quote;
use proc_macro2::{ Span, Ident, TokenStream };

/// Gera o *match pattern* de uma instrução do tipo R.
fn generate_r_kind((name, _instr): (&String, &RInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        &Instruction:: #ename_ident (_) => Kind::R,
    };

    code.into()
}


/// Gera o *match pattern* de uma instrução do tipo I.
fn generate_i_kind((name, _instr): (&String, &IInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        &Instruction:: #ename_ident (_) => Kind::I,
    };

    code.into()
}

/// Gera o *match pattern* de uma instrução do tipo J.
fn generate_j_kind((name, _instr): (&String, &JInstruction)) -> TokenStream {
    let ename = name.to_uppercase();
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        &Instruction:: #ename_ident (_) => Kind::J,
    };

    code.into()
}

/// Gera o *match pattern* de uma instrução do tipo FR.
fn generate_fr_kind((name, _instr): (&String, &FRInstruction)) -> TokenStream {
    let ename = name.to_uppercase().replace(".", "_");
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        &Instruction:: #ename_ident (_) => Kind::FR,
    };

    code.into()
}

/// Gera o *match pattern* de uma instrução do tipo FI.
fn generate_fi_kind((name, _instr): (&String, &FIInstruction)) -> TokenStream {
    let ename = name.to_uppercase().replace(".", "_");
    let ename_ident = Ident::new(&ename, Span::call_site());

    let code = quote! {
        &Instruction:: #ename_ident (_) => Kind::FI,
    };

    code.into()
}

/// Gera a implementacao de `kind()` para cada instrucao.
pub(crate) fn generate_kind(instrs: &Instructions) -> TokenStream {
    let r = instrs.r
        .iter()
        .map(generate_r_kind)
        .collect::<Vec<_>>();
    let i = instrs.i
        .iter()
        .map(generate_i_kind)
        .collect::<Vec<_>>();
    let j = instrs.j
        .iter()
        .map(generate_j_kind)
        .collect::<Vec<_>>();
    let fr = instrs.fr
        .iter()
        .map(generate_fr_kind)
        .collect::<Vec<_>>();
    let fi = instrs.fi
        .iter()
        .map(generate_fi_kind)
        .collect::<Vec<_>>();

    let code = quote! {
        /// Retorna o tipo da instrução.
        pub fn kind(&self) -> Kind {
            match self {
                &Instruction::NOP => Kind::R,
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
            }
        }
    }.into();

    code
}
