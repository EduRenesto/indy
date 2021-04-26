use color_eyre::eyre::Result;

pub mod cache;
pub mod ram;
pub mod reporter;

// TODO remover depois da integracao
pub use cache::{Cache, RepPolicy};
pub use ram::Ram;

use std::cell::UnsafeCell;

pub trait Memory {
    /// Lê o valor armazenado no endereço `addr`.
    fn peek(&mut self, addr: u32) -> Result<u32>;

    /// Escreve o valor `val` no endereço `addr`.
    fn poke(&mut self, addr: u32, val: u32) -> Result<()>;

    /// Mostra o conteúdo desse nível de memória. Apenas para debugging.
    fn dump(&self) -> Result<()>;

    /// Faz uma leitura não alinhada na memória. Isto é, retorna apenas um byte
    /// de uma word.
    fn peek_unaligned(&mut self, addr: u32) -> Result<u8> {
        let base = addr & 0xFFFFFFFC; // alinha pro lowest multiplo de 4
        let offset = addr - base; // offset agora armazena qual é o byte desejado

        let word = self.peek(base)?;

        //Ok(((word & (0xFF << offset )) >> offset) as u8)
        Ok(word.to_le_bytes()[offset as usize])
    }

    /// Carrega um bloco de dados na memória a partir do endereço especificado.
    fn load_slice_into_addr(&mut self, base: u32, data: &[u32]) -> Result<()> {
        let mut addr = base;
        for word in data {
            self.poke(addr, *word)?;
            addr += 4;
        }

        Ok(())
    }
}

impl<'a, T: Memory> Memory for &'a UnsafeCell<T> {
    fn peek(&mut self, addr: u32) -> Result<u32> {
        unsafe {
            (&mut *self.get()).peek(addr)
        }
    }

    fn poke(&mut self, addr: u32, val: u32) -> Result<()> {
        unsafe {
            (&mut *self.get()).poke(addr, val)
        }
    }

    fn dump(&self) -> Result<()> {
        unsafe {
            (&*self.get()).dump()
        }
    }
}
