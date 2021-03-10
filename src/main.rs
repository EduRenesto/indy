//! minips-rs: um emulador de MINIPS em Rust!
//!
//! O arquivo [main.rs](src/main.rs) toma conta apenas do CLI do emulador.
//! Implementação do emulador em si está no módulo `emulator`.

use clap::{crate_version, App, Arg, SubCommand};
use color_eyre::eyre::Result;
use goblin::elf::Elf;

use std::fs::File;
use std::io::Read;

pub(crate) mod emulator;

use emulator::Cpu;
use emulator::Instruction;

/// Carrega o arquivo num vetor de palavras de 32 bits.
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

/// Encapsula uma dupla .text/.data de palavras de 32 bits.
#[derive(Debug)]
pub struct Executable {
    pub text: Vec<u32>,
    pub data: Option<Vec<u32>>,
}

impl Executable {
    /// Lê os arquivos `pfx.text` e `pfx.data` e retorna um Executable contendo os dados
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
                .arg(
                    Arg::with_name("entry")
                        .long("entry")
                        .short("e")
                        .required(false)
                        .default_value("0x00400000")
                        .help("Endereço da primeira instrução"),
                )
                .arg(Arg::with_name("file").index(1).required(true)),
        )
        .subcommand(
            SubCommand::with_name("runelf")
                .about("Carrega um arquivo ELF e o executa (bonus!)")
                .arg(Arg::with_name("file").index(1).required(true)),
        )
        .get_matches();

    if let Some(matches) = matches.subcommand_matches("decode") {
        let executable = Executable::from_naked_files(matches.value_of("file").unwrap())?;

        for word in executable.text {
            println!("{}", Instruction::decode(word)?);
        }

        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("run") {
        let entry = u32::from_str_radix(&matches.value_of("entry").unwrap()[2..], 16)?;
        let executable = Executable::from_naked_files(matches.value_of("file").unwrap())?;

        //let mut cpu = Cpu::new(0x00400000, 0x7FFFEFFC, 0x10008000);
        let mut cpu = Cpu::new(entry, 0x7FFFEFFC, 0x10008000);

        cpu.memory_mut()
            .load_slice_into_addr(0x00400000, &executable.text[..])?;
        if let Some(ref data) = executable.data {
            cpu.memory_mut()
                .load_slice_into_addr(0x10010000, &data[..])?;
        }

        cpu.run()?;

        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("runelf") {
        let mut file = File::open(matches.value_of("file").unwrap())?;
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes)?;

        let elf = Elf::parse(&file_bytes[..])?;

        let mut cpu = Cpu::new(elf.entry as u32, 0x7FFFEFFC, 0x10008000);

        for section in elf.program_headers {
            if section.p_type == goblin::elf::program_header::PT_LOAD {
                println!(
                    "elf: loading {} bytes to {:#010x}...",
                    section.p_memsz, section.p_paddr
                );
                let offset = section.p_offset as usize;
                let size = section.p_filesz as usize;

                //let mut section_bytes = Vec::with_capacity(size);
                //section_bytes.copy_from_slice(&file_bytes[offset..offset + size]);
                let section_bytes: Vec<u32> = file_bytes[offset..offset + size]
                    .chunks(4)
                    .map(|b| {
                        let mut owned_b = [0u8; 4];
                        owned_b.copy_from_slice(b);
                        u32::from_le_bytes(owned_b)
                    })
                    .collect();

                cpu.memory_mut()
                    .load_slice_into_addr(section.p_paddr as u32, &section_bytes[..])?;
            }
        }

        cpu.run()?;

        Ok(())
    } else {
        eprintln!("{}", matches.usage());
        Ok(())
    }
}
