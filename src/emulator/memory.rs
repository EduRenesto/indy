use color_eyre::eyre::{Result, eyre};

use std::collections::HashMap;

macro_rules! check_alignment {
    ($addr:ident) => {
        if $addr % 4 != 0 {
            return Err(eyre!("Acesso a endereço não alinhado! {:x}", $addr));
        }
    }
}

/// Como sugerido, a memória é só um HashMap onde as chaves são os endereços.
pub struct Memory {
    memory: HashMap<u32, u32>,
}

impl Memory {
    /// Cria um novo objeto Memory, com capacidade pré-alocada de 1024 words.
    pub fn new() -> Memory {
        Memory {
            memory: HashMap::with_capacity(1024)
        }
    }

    /// Retorna o valor no endereço especificado, sendo 0 caso não tenha sido
    /// inicalizado.
    pub fn peek(&self, addr: u32) -> Result<&u32> {
        check_alignment!(addr);

        Ok(self.memory.get(&addr).unwrap_or(&0))
    }

    /// Modifica um valor no endereço especificado.
    pub fn poke(&mut self, addr: u32, val: u32) -> Result<()> {
        check_alignment!(addr);

        self.memory.insert(addr, val);

        Ok(())
    }

    /// Carrega um bloco de dados na memória a partir do endereço especificado.
    pub fn load_slice_into_addr(&mut self, addr: u32, data: &[u32]) -> Result<()> {
        check_alignment!(addr);

        let mut addr = addr;
        for word in data {
            self.poke(addr, *word)?;
            addr += 4;
        }

        Ok(())
    }
}
