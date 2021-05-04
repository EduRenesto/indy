//! Esse módulo implementa as funções relacionadas à memória no emulador.
//! Futuramente, quero implementar uma MMU para mapping e caches, e
//! provavelmente boa parte dessa empreitada aparecerá nesse módulo.

use super::Memory;

use color_eyre::eyre::{eyre, Result};
use log::debug;

use std::collections::HashMap;

/// Checa o alinhamento de um endereço. Causa erro caso não seja alinhado a
/// palavras.
macro_rules! check_alignment {
    ($addr:ident) => {
        if $addr % 4 != 0 {
            return Err(eyre!("Acesso a endereço não alinhado! {:#x}", $addr));
        }
    };
}

/// Como sugerido, a memória é só um HashMap onde as chaves são os endereços.
pub struct Ram {
    memory: HashMap<u32, u32>,
    latency: usize,
    accesses: usize,
}

impl Ram {
    /// Cria um novo objeto Ram, com capacidade pré-alocada de 1024 words.
    pub fn new(latency: usize) -> Ram {
        Ram {
            memory: HashMap::with_capacity(1024),
            latency,
            accesses: 0,
        }
    }

    /// Reseta a contagem de acessos.
    pub fn reset_stats(&mut self) {
        self.accesses = 0;
    }
}

impl Memory for Ram {
    /// Retorna o valor no endereço especificado, sendo 0 caso não tenha sido
    /// inicalizado.
    fn peek(&mut self, addr: u32) -> Result<(u32, usize)> {
        check_alignment!(addr);

        self.accesses += 1;

        //println!("mem: read {:#010x}", addr);

        Ok((*self.memory.get(&addr).unwrap_or(&0), self.latency))
    }

    fn peek_instruction(&mut self, addr: u32) -> Result<(u32, usize)> {
        check_alignment!(addr);

        self.accesses += 1;

        //println!("mem: read {:#010x}", addr);

        Ok((*self.memory.get(&addr).unwrap_or(&0), self.latency))
    }

    #[allow(clippy::needless_range_loop)]
    fn peek_into_slice(&mut self, addr: u32, target: &mut [u32]) -> Result<usize> {
        check_alignment!(addr);

        self.accesses += 1;

        for i in 0..target.len() {
            let target_addr = addr + 4 * i as u32;
            target[i] = *self.memory.get(&target_addr).unwrap_or(&0);
            debug!("ram: target[{}] <- {:#010x}", i, target_addr);
        }

        Ok(self.latency)
    }

    /// Modifica um valor no endereço especificado.
    fn poke(&mut self, addr: u32, val: u32) -> Result<usize> {
        check_alignment!(addr);

        self.accesses += 1;

        //println!("mem: write {:#010x}", addr);

        self.memory.insert(addr, val);

        Ok(self.latency)
    }

    fn poke_from_slice(&mut self, base: u32, data: &[u32]) -> Result<usize> {
        let mut addr = base;
        for word in data {
            //self.poke(addr, *word)?;
            self.memory.insert(addr, *word);
            addr += 4;
        }

        self.accesses += 1;

        Ok(self.latency)
    }

    /// Faz uma leitura não alinhada na memória. Isto é, retorna apenas um byte
    /// de uma word.
    fn peek_unaligned(&mut self, addr: u32) -> Result<u8> {
        let base = addr & 0xFFFFFFFC; // alinha pro lowest multiplo de 4
        let offset = addr - base; // offset agora armazena qual é o byte desejado

        let word = self.peek(base)?.0;

        self.accesses += 1;

        //Ok(((word & (0xFF << offset )) >> offset) as u8)
        Ok(word.to_le_bytes()[offset as usize])
    }

    fn dump(&self) -> Result<()> {
        unimplemented!()
    }

    fn print_stats(&self, _: bool) {
        println!(
            "{:>5}  {:>12}  {:>12}  {:>12}   {:>8.2}%",
            "RAM", self.accesses, 0, self.accesses, 0.0
        );
    }
}
