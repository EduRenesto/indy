//! Reportagem de uso de memória.

use std::fs::File;
use std::io::Write;
use std::sync::mpsc;
use std::thread;

/// As mensagens que o Memory Reporter pode receber.
#[derive(Clone, Debug)]
pub enum MemoryEvent {
    /// Uma leitura de dados. O primeiro elemento é o endereço, o segundo é a linha.
    DataRead(u32, usize),
    /// Uma leitura de instrução. O primeiro elemento é o endereço, o segundo é a linha.
    InstrRead(u32, usize),
    /// Uma escrita. O primeiro elemento é o endereço, o segundo é a linha.
    Write(u32, usize),
    /// Texto comum, que só será impresso quando o modo `debug` estiver habilitado.
    Debug(String),
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
    #[allow(clippy::new_ret_no_self)]
    pub fn new(file: File, debug: bool) -> (thread::JoinHandle<()>, mpsc::SyncSender<MemoryEvent>) {
        // Quanto maior o valor aqui, mais rápido o `trace` e `debug` vão rodar.
        // No entanto, vai consumir mais memória.
        let (tx, rx) = mpsc::sync_channel(1000000);

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
                    MemoryEvent::Debug(text) => {
                        if debug {
                            writeln!(file, "{}", text).unwrap();
                        }
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
