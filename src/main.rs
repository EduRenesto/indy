//! minips-rs: um emulador de MINIPS em Rust!
//!
//! O arquivo [main.rs](src/main.rs) toma conta apenas do CLI do emulador.
//! Implementação do emulador em si está no módulo `emulator`.

use clap::{crate_version, App, Arg, SubCommand};
use color_eyre::eyre::Result;

use std::io::Read;
use std::{error::Error, fs::File};

pub(crate) mod emulator;

fn u32_vec_from_file(mut file: File) -> Vec<u32> {
    let mut data = Vec::new();
    file.read_to_end(&mut data).unwrap();

    data.chunks(4)
        .map(|b| {
            let mut owned_b = [0u8; 4];
            owned_b.copy_from_slice(b);
            u32::from_le_bytes(owned_b)
        })
        .collect()
}

#[derive(Debug)]
pub struct Executable {
    pub text: Vec<u32>,
    pub data: Option<Vec<u32>>,
}

impl Executable {
    pub fn from_naked_files(pfx: impl AsRef<str>) -> Result<Executable> {
        let text = u32_vec_from_file(File::open(format!("{}.text", pfx.as_ref()))?);
        let data = File::open(format!("{}.data", pfx.as_ref()))
            .ok()
            .map(u32_vec_from_file);

        Ok(Executable { text, data })
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    let matches = App::new("minips-rs")
        .version(crate_version!())
        .author("Edu Renesto, eduardo.renesto@aluno.ufabc.edu.br")
        .subcommand(
            SubCommand::with_name("decode")
                .about("Desconstrói o binário, mostrando o código Assembly equivalente")
                .arg(Arg::with_name("file").index(1).required(true)),
        )
        .subcommand(
            SubCommand::with_name("run")
                .about("Carrega o binário e o executa")
                .arg(Arg::with_name("file").index(1).required(true)),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("decode") {
        let executable = Executable::from_naked_files(matches.value_of("file").unwrap())?;
        println!("{:?}", executable);
        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("run") {
        let executable = Executable::from_naked_files(matches.value_of("file").unwrap())?;
        println!("{:?}", executable);
        Ok(())
    } else {
        eprintln!("{}", matches.usage());
        Ok(())
    }
}
