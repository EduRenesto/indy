//! indy: um emulador de MINIPS em Rust!
//!
//! O arquivo [main.rs](src/main.rs) toma conta apenas do CLI do emulador.
//! Implementação do emulador em si está no módulo `emulator`.

use clap::{crate_version, App, Arg, SubCommand};
use color_eyre::eyre::{eyre, Result};
use goblin::elf::Elf;

use std::cell::UnsafeCell;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::SyncSender;

pub(crate) mod emulator;

use emulator::memory::{reporter::*, Cache, Memory, Ram, RepPolicy};
use emulator::Cpu;
use emulator::Instruction;

/// Descrição e tabela das configurações de memória disponíveis.
const CONFIG_HELP: &str = "As configurações de memória podem ser as seguintes:
(tabela copiada do `indy` de referência)

|------|--------|-----------|-------------|--------|------------|-----------|
| Conf | Níveis | Tipo      | Tamanho     | Map.   | Tam./Linha | Política  |
|------|--------|-----------|-------------|--------|------------|-----------|
| 1    | 0      | -         | -           | -      | -          | -         |
|------|--------|-----------|-------------|--------|------------|-----------|
| 2    | 1      | Unificada | 1024        | Direto | 32         | Aleatória |
|------|--------|-----------|-------------|--------|------------|-----------|
| 3    | 1      | Split     | 512/cada    | Direto | 32         | Aleatória |
|------|--------|-----------|-------------|--------|------------|-----------|
| 4    | 1      | Split     | 512/cada    | Direto | 32         | LRU       |
|------|--------|-----------|-------------|--------|------------|-----------|
| 5    | 1      | Split     | 512/cada    | 4 vias | 32         | LRU       |
|------|--------|-----------|-------------|--------|------------|-----------|
| 6    | 2      | Split     | L1 512/cada | 4 vias | 64         | LRU       |
|      |        | Unificada | L2 2048     | 8 vias | 64         | LRU       |
|------|--------|-----------|-------------|--------|------------|-----------|

Se não informada, a configuração 1 é a padrão.
";

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

/// Assume que o objeto `ram` recebido já contém o conteúdo do programa
/// a ser executado, e o executa levando em consideração a configuração
/// de memória e o reporter associado.
fn run_from_ram(
    ram: Ram,
    entry: u32,
    mem_cfg: &str,
    tx: Option<SyncSender<MemoryEvent>>,
) -> Result<()> {
    match mem_cfg {
        "1" => {
            let ram = UnsafeCell::new(ram);
            let mut cpu = Cpu::new(&ram, &ram, entry, 0x7FFFEFFC, 0x10008000);
            cpu.run()?;
        }
        "2" => {
            let ram = UnsafeCell::new(ram);
            let cache: UnsafeCell<Cache<_, 8, 32, 1>> = UnsafeCell::new(Cache::new(
                "L1",
                &ram,
                RepPolicy::Random,
                1,
                tx.as_ref().cloned(),
            ));
            let mut cpu = Cpu::new(&cache, &cache, entry, 0x7FFFEFFC, 0x10008000);
            cpu.run()?;
        }
        "3" => {
            let ram = UnsafeCell::new(ram);
            let l1d: UnsafeCell<Cache<_, 8, 16, 1>> = UnsafeCell::new(Cache::new(
                "L1d",
                &ram,
                RepPolicy::Random,
                1,
                tx.as_ref().cloned(),
            ));
            let l1i: UnsafeCell<Cache<_, 8, 16, 1>> = UnsafeCell::new(Cache::new(
                "L1i",
                &ram,
                RepPolicy::Random,
                1,
                tx.as_ref().cloned(),
            ));

            unsafe {
                (&mut *l1d.get()).set_sister(&l1i, true);
                (&mut *l1i.get()).set_sister(&l1d, true);
            }

            let mut cpu = Cpu::new(&l1d, &l1i, entry, 0x7FFFEFFC, 0x10008000);
            cpu.run()?;
        }
        "4" => {
            let ram = UnsafeCell::new(ram);
            let l1d: UnsafeCell<Cache<_, 8, 16, 1>> = UnsafeCell::new(Cache::new(
                "L1d",
                &ram,
                RepPolicy::LeastRecentlyUsed,
                1,
                tx.as_ref().cloned(),
            ));
            let l1i: UnsafeCell<Cache<_, 8, 16, 1>> = UnsafeCell::new(Cache::new(
                "L1i",
                &ram,
                RepPolicy::LeastRecentlyUsed,
                1,
                tx.as_ref().cloned(),
            ));

            unsafe {
                (&mut *l1d.get()).set_sister(&l1i, true);
                (&mut *l1i.get()).set_sister(&l1d, true);
            }

            let mut cpu = Cpu::new(&l1d, &l1i, entry, 0x7FFFEFFC, 0x10008000);
            cpu.run()?;
        }
        "5" => {
            let ram = UnsafeCell::new(ram);
            let l1d: UnsafeCell<Cache<_, 8, 16, 4>> = UnsafeCell::new(Cache::new(
                "L1d",
                &ram,
                RepPolicy::LeastRecentlyUsed,
                1,
                tx.as_ref().cloned(),
            ));
            let l1i: UnsafeCell<Cache<_, 8, 16, 4>> = UnsafeCell::new(Cache::new(
                "L1i",
                &ram,
                RepPolicy::LeastRecentlyUsed,
                1,
                tx.as_ref().cloned(),
            ));

            unsafe {
                (&mut *l1d.get()).set_sister(&l1i, true);
                (&mut *l1i.get()).set_sister(&l1d, true);
            }

            let mut cpu = Cpu::new(&l1d, &l1i, entry, 0x7FFFEFFC, 0x10008000);
            cpu.run()?;
        }
        "6" => {
            let ram = UnsafeCell::new(ram);

            let l2: UnsafeCell<Cache<_, 16, 32, 8>> = UnsafeCell::new(Cache::new(
                "L2",
                &ram,
                RepPolicy::LeastRecentlyUsed,
                10,
                None,
            ));

            let l1d: UnsafeCell<Cache<_, 16, 8, 4>> = UnsafeCell::new(Cache::new(
                "L1d",
                &l2,
                RepPolicy::LeastRecentlyUsed,
                1,
                tx.as_ref().cloned(),
            ));
            let l1i: UnsafeCell<Cache<_, 16, 8, 4>> = UnsafeCell::new(Cache::new(
                "L1i",
                &l2,
                RepPolicy::LeastRecentlyUsed,
                1,
                tx.as_ref().cloned(),
            ));

            unsafe {
                (&mut *l1d.get()).set_sister(&l1i, true);
                (&mut *l1i.get()).set_sister(&l1d, true);
            }

            let mut cpu = Cpu::new(&l1d, &l1i, entry, 0x7FFFEFFC, 0x10008000);
            cpu.run()?;
        }
        c => return Err(eyre!("Configuração de memória {} não conhecida!", c)),
    };
    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;

    env_logger::init();

    // Aqui é descrito o CLI do emulador.
    // Não vou comentar porque a API do clap é bem auto-descritiva
    let matches = App::new("indy")
        .version(crate_version!())
        .author("Edu Renesto <eduardo.renesto@aluno.ufabc.edu.br>")
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
                .arg(Arg::with_name("conf").required(true).index(1).help("Índice da configuração da memória").long_help(CONFIG_HELP))
                .arg(Arg::with_name("file").required(false).index(2).help("O caminho do programa a ser executado")),
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
                        .default_value("indy.trace")
                        .help("Arquivo onde escrever os acessos de memória"),
                )
                .arg(Arg::with_name("conf").required(true).index(1).help("Índice da configuração da memória").long_help(CONFIG_HELP))
                .arg(Arg::with_name("file").required(false).index(2).help("O caminho do programa a ser executado")),
        )
        .subcommand(
            SubCommand::with_name("debug")
                .about("Carrega o binário e o executa, escrevendo os acessos de memória e outras infos no arquivo.")
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
                        .default_value("indy.trace")
                        .help("Arquivo onde escrever os acessos de memória"),
                )
                .arg(Arg::with_name("conf").required(true).index(1).help("Índice da configuração da memória").long_help(CONFIG_HELP))
                .arg(Arg::with_name("file").required(false).index(2).help("O caminho do programa a ser executado")),
        )
        .subcommand(
            SubCommand::with_name("runelf")
                .about("Carrega um arquivo ELF e o executa (bonus!)")
                .arg(Arg::with_name("conf").required(true).index(1).help("Índice da configuração da memória").long_help(CONFIG_HELP))
                .arg(Arg::with_name("file").required(false).index(2).help("O caminho do programa a ser executado")),
        )
        .subcommand(
            SubCommand::with_name("decodeelf")
                .about("Carrega um arquivo ELF e o desconstrói, mostrando o código Assembly equivalente (bonus!)")
                .arg(Arg::with_name("file").required(true)),
        )
        .get_matches();

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
        let (file, mem_cfg) = match matches.value_of("file") {
            Some(file) => (file, matches.value_of("conf").unwrap()),
            None => (matches.value_of("conf").unwrap(), "1"),
        };

        let mut ram = Ram::new(100);

        // Executa o binário
        let entry = u32::from_str_radix(&matches.value_of("entry").unwrap()[2..], 16)?;
        let executable = Executable::from_naked_files(file)?;

        //let mut cpu = Cpu::new(0x00400000, 0x7FFFEFFC, 0x10008000);

        ram.poke_from_slice(0x00400000, &executable.text[..])?;
        if let Some(ref data) = executable.data {
            ram.poke_from_slice(0x10010000, &data[..])?;
        }
        if let Some(ref data) = executable.rodata {
            ram.poke_from_slice(0x00800000, &data[..])?;
        }

        ram.reset_stats();

        run_from_ram(ram, entry, mem_cfg, None)?;

        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("trace") {
        let (file, mem_cfg) = match matches.value_of("file") {
            Some(file) => (file, matches.value_of("conf").unwrap()),
            None => (matches.value_of("conf").unwrap(), "1"),
        };

        let mut ram = Ram::new(100);

        // Executa o binário
        let entry = u32::from_str_radix(&matches.value_of("entry").unwrap()[2..], 16)?;
        let executable = Executable::from_naked_files(file)?;

        //let mut cpu = Cpu::new(0x00400000, 0x7FFFEFFC, 0x10008000);

        ram.poke_from_slice(0x00400000, &executable.text[..])?;
        if let Some(ref data) = executable.data {
            ram.poke_from_slice(0x10010000, &data[..])?;
        }
        if let Some(ref data) = executable.rodata {
            ram.poke_from_slice(0x00800000, &data[..])?;
        }

        ram.reset_stats();

        let out_file = matches.value_of("outfile").unwrap();
        let out_file = File::create(out_file)?;
        let (rep_thread, tx) = MemoryReporter::new(out_file, false);

        run_from_ram(ram, entry, mem_cfg, Some(tx.clone()))?;

        tx.send(MemoryEvent::Finish).unwrap();
        rep_thread.join().unwrap();

        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("debug") {
        let (file, mem_cfg) = match matches.value_of("file") {
            Some(file) => (file, matches.value_of("conf").unwrap()),
            None => (matches.value_of("conf").unwrap(), "1"),
        };

        let mut ram = Ram::new(100);

        // Executa o binário
        let entry = u32::from_str_radix(&matches.value_of("entry").unwrap()[2..], 16)?;
        let executable = Executable::from_naked_files(file)?;

        //let mut cpu = Cpu::new(0x00400000, 0x7FFFEFFC, 0x10008000);

        ram.poke_from_slice(0x00400000, &executable.text[..])?;
        if let Some(ref data) = executable.data {
            ram.poke_from_slice(0x10010000, &data[..])?;
        }
        if let Some(ref data) = executable.rodata {
            ram.poke_from_slice(0x00800000, &data[..])?;
        }

        ram.reset_stats();

        let out_file = matches.value_of("outfile").unwrap();
        let out_file = File::create(out_file)?;
        let (rep_thread, tx) = MemoryReporter::new(out_file, true);

        run_from_ram(ram, entry, mem_cfg, Some(tx.clone()))?;

        tx.send(MemoryEvent::Finish).unwrap();
        rep_thread.join().unwrap();

        Ok(())
    } else if let Some(matches) = matches.subcommand_matches("runelf") {
        let (file, mem_cfg) = match matches.value_of("file") {
            Some(file) => (file, matches.value_of("conf").unwrap()),
            None => (matches.value_of("conf").unwrap(), "1"),
        };

        let mut ram = Ram::new(100);

        // Executa o elf
        let mut file = File::open(file)?;
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

                ram.poke_from_slice(section.p_paddr as u32, &section_bytes[..])?;
            }
        }

        ram.reset_stats();

        run_from_ram(ram, elf.entry as u32, mem_cfg, None)?;

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
                        Ok(instr) => print!("{:08x}:\t{:08x}\t{}", offset + pos, word, instr),
                        Err(_) => print!("{:08x}:\t{:08x}\t???", word, offset + pos),
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
        eprintln!("Tente: indy --help");
        Ok(())
    }
}
