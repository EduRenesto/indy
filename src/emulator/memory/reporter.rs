//! TODO Transformar isso numa trait e usar o type system
//! pra quando não é preciso!
//!
//! Pelo jeito, perdi um pouco de performance nas caches
//! ao fazer a checagem se existe um sender a cada leitura
//! e escrita.
//!
//! Se fazer isso pelo type system, minha ideia é que no
//! final sejam gerados NOP no modo run, e assim o compilador
//! pode só otimizar pra fora.

use std::fs::File;
use std::io::Write;
use std::sync::mpsc;
use std::thread;

/// As mensagens que o Memory Reporter pode receber.
#[derive(Copy, Clone, Debug)]
pub enum MemoryEvent {
    /// Uma leitura de dados. O primeiro elemento é o endereço, o segundo é a linha.
    DataRead(u32, usize),
    /// Uma leitura de instrução. O primeiro elemento é o endereço, o segundo é a linha.
    InstrRead(u32, usize),
    /// Uma escrita. O primeiro elemento é o endereço, o segundo é a linha.
    Write(u32, usize),
    /// Finaliza o Reporter.
    Finish,
}

/// O `MemoryReporter` spawna uma thread nova que escreve num arquivo
/// os eventos de memória.
/// Isso podia ser feito usando um BufferedReader nas próprias implementações
/// da trait `Memory`, mas de qualquer modo iriam haver ciclos onde seria gasto
/// tempo para esvaziar o buffer. Usando uma outra thread, a thread principal fica
/// tranquila prá só fazer a emulação.
pub struct MemoryReporter;

impl MemoryReporter {
    /// Cria um novo `MemoryReporter`, iniciando a thread e retornando o join handle
    /// da mesma, e o write end do channel.
    pub fn new(file: File) -> (thread::JoinHandle<()>, mpsc::Sender<MemoryEvent>) {
        let (tx, rx) = mpsc::channel();

        let handle = thread::spawn(move || {
            let mut file = file;
            while let Ok(msg) = rx.recv() {
                match msg {
                    MemoryEvent::DataRead(addr, line) => {
                        writeln!(file, "R {:#010x} (line# {:#010x})", addr, line).unwrap();
                    }
                    MemoryEvent::InstrRead(addr, line) => {
                        writeln!(file, "I {:#010x} (line# {:#010x})", addr, line).unwrap();
                    }
                    MemoryEvent::Write(addr, line) => {
                        writeln!(file, "W {:#010x} (line# {:#010x})", addr, line).unwrap();
                    }
                    MemoryEvent::Finish => {
                        file.flush().unwrap();
                        break;
                    }
                }
            }
        });

        (handle, tx)
    }
}
