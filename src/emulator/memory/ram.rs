//! Esse módulo implementa as funções relacionadas à memória no emulador.
//! Futuramente, quero implementar uma MMU para mapping e caches, e
//! provavelmente boa parte dessa empreitada aparecerá nesse módulo.

use super::Memory;

use color_eyre::eyre::{eyre, Result};

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
}

impl Ram {
    /// Cria um novo objeto Ram, com capacidade pré-alocada de 1024 words.
    pub fn new() -> Ram {
        Ram {
            memory: HashMap::with_capacity(1024),
        }
    }
}

impl Memory for Ram {
    /// Retorna o valor no endereço especificado, sendo 0 caso não tenha sido
    /// inicalizado.
    fn peek(&mut self, addr: u32) -> Result<u32> {
        check_alignment!(addr);

        //println!("mem: read {:#010x}", addr);

        Ok(*self.memory.get(&addr).unwrap_or(&0))
    }

    /// Modifica um valor no endereço especificado.
    fn poke(&mut self, addr: u32, val: u32) -> Result<()> {
        check_alignment!(addr);

        //println!("mem: write {:#010x}", addr);

        self.memory.insert(addr, val);

        Ok(())
    }

    /// Faz uma leitura não alinhada na memória. Isto é, retorna apenas um byte
    /// de uma word.
    fn peek_unaligned(&mut self, addr: u32) -> Result<u8> {
        let base = addr & 0xFFFFFFFC; // alinha pro lowest multiplo de 4
        let offset = addr - base; // offset agora armazena qual é o byte desejado

        let word = self.peek(base)?;

        //Ok(((word & (0xFF << offset )) >> offset) as u8)
        Ok(word.to_le_bytes()[offset as usize])
    }

    fn dump(&self) -> Result<()> {
        unimplemented!()
    }
}
