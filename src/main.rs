//! minips-rs: um emulador de MINIPS em Rust!
//!
//! O arquivo [main.rs](src/main.rs) toma conta apenas do CLI do emulador.
//! Implementação do emulador em si está no módulo `emulator`.

use clap::{crate_version, App, Arg, SubCommand};
use color_eyre::eyre::{eyre, Result};
use goblin::elf::Elf;

use std::cell::UnsafeCell;
use std::fs::File;
use std::io::Read;

pub(crate) mod emulator;

use emulator::memory::{reporter::*, Cache, Memory, Ram, RepPolicy};
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
    pub rodata: Option<Vec<u32>>,
}

impl Executable {
    /// Lê os arquivos `pfx.text` e `pfx.data` e retorna um Executable contendo os dados
    pub fn from_naked_files(pfx: impl AsRef<str>) -> Result<Executable> {
        let text = u32_vec_from_file(File::open(format!("{}.text", pfx.as_ref()))?);
        let data = File::open(format!("{}.data", pfx.as_ref()))
            .ok()
            .map(u32_vec_from_file);
        let rodata = File::open(format!("{}.rodata", pfx.as_ref()))
            .ok()
            .map(u32_vec_from_file);

        Ok(Executable { text, data, rodata })
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;

    //simple_logger::SimpleLogger::new().init().unwrap();

    // Aqui é descrito o CLI do emulador.
    // Não vou comentar porque a API do clap é bem auto-descritiva
    let matches = App::new("minips-rs")
        .version(crate_version!())
        .author("Edu Renesto, eduardo.renesto@aluno.ufabc.edu.br")
        .arg(Arg::with_name("conf").required(false).default_value("1").help("Índice da configuração da memória"))
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
            SubCommand::with_name("trace")
                .about("Carrega o binário e o executa, escrevendo os acessos de memória no arquivo.")
                .arg(
                    Arg::with_name("entry")
                        .long("entry")
                        .short("e")
                        .required(false)
                        .default_value("0x00400000")
                        .help("Endereço da primeira instrução"),
                )
                .arg(
                    Arg::with_name("outfile")
                        .long("outfile")
                        .short("o")
                        .required(false)
                        .default_value("minips.trace")
                        .help("Arquivo onde escrever os acessos de memória"),
                )
                .arg(Arg::with_name("file").index(1).required(true)),
        )
        .subcommand(
            SubCommand::with_name("runelf")
                .about("Carrega um arquivo ELF e o executa (bonus!)")
                .arg(Arg::with_name("file").index(1).required(true)),
        )
        .subcommand(
            SubCommand::with_name("decodeelf")
                .about("Carrega um arquivo ELF e o desconstrói, mostrando o código Assembly equivalente (bonus!)")
                .arg(Arg::with_name("file").index(1).required(true)),
        )
        .get_matches();

    let mem_cfg = matches.value_of("conf").unwrap();

    if let Some(matches) = matches.subcommand_matches("decode") {
        // Desmonta o binário
        let executable = Executable::from_naked_files(matches.value_of("file").unwrap())?;

        let mut addr = 0x00400000;

        for word in executable.text {
            println!(
                "{:08x}:\t{:08x}\t{}",
                addr,
                word,
                Instruction::decode(word)?
            );
            addr += 4;
        }

        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("run") {
        let mut ram = Ram::new();

        // Executa o binário
        let entry = u32::from_str_radix(&matches.value_of("entry").unwrap()[2..], 16)?;
        let executable = Executable::from_naked_files(matches.value_of("file").unwrap())?;

        //let mut cpu = Cpu::new(0x00400000, 0x7FFFEFFC, 0x10008000);

        ram.load_slice_into_addr(0x00400000, &executable.text[..])?;
        if let Some(ref data) = executable.data {
            ram.load_slice_into_addr(0x10010000, &data[..])?;
        }
        if let Some(ref data) = executable.rodata {
            ram.load_slice_into_addr(0x00800000, &data[..])?;
        }

        match mem_cfg {
            "1" => {
                let ram = UnsafeCell::new(ram);
                let mut cpu = Cpu::new(&ram, &ram, entry, 0x7FFFEFFC, 0x10008000);
                cpu.run()?;
            }
            "2" => {
                let ram = UnsafeCell::new(ram);
                let cache: UnsafeCell<Cache<_, 1, 1024, 1>> =
                    UnsafeCell::new(Cache::new("L1", &ram, RepPolicy::Random, 1, None));
                let mut cpu = Cpu::new(&cache, &cache, entry, 0x7FFFEFFC, 0x10008000);
                cpu.run()?;
            }
            c => return Err(eyre!("Configuração de memória {} não conhecida!", c)),
        };

        //let cache: Cache<_, 8, 1024, 2> = Cache::new("L1", &ram, RepPolicy::Random, 1);

        //let mut cpu = Cpu::new(cache, entry, 0x7FFFEFFC, 0x10008000);

        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("trace") {
        let mut ram = Ram::new();

        // Executa o binário
        let entry = u32::from_str_radix(&matches.value_of("entry").unwrap()[2..], 16)?;
        let executable = Executable::from_naked_files(matches.value_of("file").unwrap())?;

        //let mut cpu = Cpu::new(0x00400000, 0x7FFFEFFC, 0x10008000);

        ram.load_slice_into_addr(0x00400000, &executable.text[..])?;
        if let Some(ref data) = executable.data {
            ram.load_slice_into_addr(0x10010000, &data[..])?;
        }
        if let Some(ref data) = executable.rodata {
            ram.load_slice_into_addr(0x00800000, &data[..])?;
        }

        let out_file = matches.value_of("outfile").unwrap();
        let out_file = File::create(out_file)?;
        let (rep_thread, tx) = MemoryReporter::new(out_file);

        match mem_cfg {
            "1" => {
                let ram = UnsafeCell::new(ram);
                let mut cpu = Cpu::new(&ram, &ram, entry, 0x7FFFEFFC, 0x10008000);
                cpu.run()?;
            }
            "2" => {
                let ram = UnsafeCell::new(ram);
                let cache: UnsafeCell<Cache<_, 1, 1024, 1>> = UnsafeCell::new(Cache::new(
                    "L1",
                    &ram,
                    RepPolicy::Random,
                    1,
                    Some(tx.clone()),
                ));
                let mut cpu = Cpu::new(&cache, &cache, entry, 0x7FFFEFFC, 0x10008000);
                cpu.run()?;
            }
            c => return Err(eyre!("Configuração de memória {} não conhecida!", c)),
        };

        tx.send(MemoryEvent::Finish).unwrap();
        rep_thread.join().unwrap();

        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("runelf") {
        let mut ram = Ram::new();

        // Executa o elf
        let mut file = File::open(matches.value_of("file").unwrap())?;
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes)?;

        let elf = Elf::parse(&file_bytes[..])?;

        // Carrega cada seção carregável em seu respectivo endereço
        for section in elf.program_headers {
            if section.p_type == goblin::elf::program_header::PT_LOAD {
                println!(
                    "elf: loading {} bytes to {:#010x}...",
                    section.p_memsz, section.p_paddr
                );
                let offset = section.p_offset as usize;
                let size = section.p_filesz as usize;

                let section_bytes: Vec<u32> = file_bytes[offset..offset + size]
                    .chunks(4)
                    .map(|b| {
                        let mut owned_b = [0u8; 4];
                        owned_b.copy_from_slice(b);
                        u32::from_le_bytes(owned_b)
                    })
                    .collect();

                ram.load_slice_into_addr(section.p_paddr as u32, &section_bytes[..])?;
            }
        }

        let ram = UnsafeCell::new(ram);
        let cache: UnsafeCell<Cache<_, 1, 8, 1>> =
            UnsafeCell::new(Cache::new("L1", &ram, RepPolicy::Random, 1, None));

        // Seta o PC para o entry point do arquivo ELF
        let mut cpu = Cpu::new(&cache, &cache, elf.entry as u32, 0x7FFFEFFC, 0x10008000);

        cpu.run()?;

        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("decodeelf") {
        // Disassemble do arquivo ELF
        let mut file = File::open(matches.value_of("file").unwrap())?;
        let mut file_bytes = Vec::new();
        file.read_to_end(&mut file_bytes)?;

        let elf = Elf::parse(&file_bytes[..])?;

        // Só desmonte as seções que vão ser carregadas na memória
        for section in elf.section_headers {
            if section.sh_type == goblin::elf::section_header::SHT_PROGBITS
                && section.sh_flags & goblin::elf::section_header::SHF_ALLOC as u64 != 0
            {
                println!(
                    "Diassemble of section {}:",
                    &elf.shdr_strtab[section.sh_name]
                );
                let offset = section.sh_offset as usize;
                let size = section.sh_size as usize;

                let section_bytes: Vec<u32> = file_bytes[offset..offset + size]
                    .chunks(4)
                    .map(|b| {
                        let mut owned_b = [0u8; 4];
                        owned_b.copy_from_slice(b);
                        u32::from_le_bytes(owned_b)
                    })
                    .collect();

                let offset = section.sh_addr;
                let mut pos = 0;
                for word in section_bytes {
                    match Instruction::decode(word) {
                        Ok(instr) => print!("{:#010x}: {}", offset + pos, instr),
                        Err(_) => print!("{:#010x}: ???", offset + pos),
                    }
                    if offset + pos == elf.entry {
                        println!(" # <- entry");
                    } else {
                        println!();
                    }
                    pos += 4;
                }
                println!();
            }
        }

        Ok(())
    } else {
        eprintln!("{}", matches.usage());
        Ok(())
    }
}
