//! # Memory
//!
//! Esse módulo implementa as estruturas que emulam acesso
//! a memória.
//!
//! A trait `Memory` representa a interface externa de um dispositivo
//! de memória qualquer, com funções de leitura e escrita, tanto única
//! quanto múltipla.
//!
//! O módulo `cache` implementa as memórias Cache.

use color_eyre::eyre::Result;

pub mod cache;
pub mod ram;
pub mod reporter;

pub use cache::{Cache, RepPolicy};
pub use ram::Ram;

use std::cell::UnsafeCell;

/// Interface geral de um dispositivo de memória.
pub trait Memory {
    /// Lê o valor armazenado no endereço `addr`. Retorna uma tupla contendo
    /// o valor e o total de ciclos gasto.
    fn peek(&mut self, addr: u32) -> Result<(u32, usize)>;

    /// Lê a instrução armazenada no endereço `addr`. Retorna uma tupla contendo
    /// o valor e o total de ciclos gasto.
    fn peek_instruction(&mut self, addr: u32) -> Result<(u32, usize)>;

    /// Copia um range de memória contíguo a partir de `addr` para
    /// `target`. Retorna o total de ciclos gasto.
    fn peek_into_slice(&mut self, addr: u32, target: &mut [u32]) -> Result<usize>;

    /// Escreve o valor `val` no endereço `addr`. Retorna o total de ciclos gasto.
    fn poke(&mut self, addr: u32, val: u32) -> Result<usize>;

    /// Carrega um bloco de dados na memória a partir do endereço especificado.
    /// Retorna o total de ciclos gasto.
    fn poke_from_slice(&mut self, base: u32, data: &[u32]) -> Result<usize>;

    /// Escreve as estatísticas de acesso na saída padrão.
    /// Se `recurse` é `true` e a memória tem outros níveis abaixo,
    /// então também mostra as estatísticas dessa.
    fn print_stats(&self, recurse: bool);

    /// Mostra o conteúdo desse nível de memória. Apenas para debugging.
    fn dump(&self) -> Result<()>;

    /// Faz uma leitura não alinhada na memória. Isto é, retorna apenas um byte
    /// de uma word.
    ///
    /// Método deprecado: faça a coisa certa e leia palavra-a-palavra1
    #[deprecated]
    fn peek_unaligned(&mut self, addr: u32) -> Result<u8> {
        let base = addr & 0xFFFFFFFC; // alinha pro lowest multiplo de 4
        let offset = addr - base; // offset agora armazena qual é o byte desejado

        let word = self.peek(base)?.0;

        Ok(word.to_le_bytes()[offset as usize])
    }
}

// LOL, eu não sabia que podia fazer isso!
// Type system lindo!
impl<'a, T: Memory> Memory for &'a UnsafeCell<T> {
    fn peek(&mut self, addr: u32) -> Result<(u32, usize)> {
        unsafe { (&mut *self.get()).peek(addr) }
    }

    fn peek_instruction(&mut self, addr: u32) -> Result<(u32, usize)> {
        unsafe { (&mut *self.get()).peek_instruction(addr) }
    }

    fn peek_into_slice(&mut self, addr: u32, target: &mut [u32]) -> Result<usize> {
        unsafe { (&mut *self.get()).peek_into_slice(addr, target) }
    }

    fn poke(&mut self, addr: u32, val: u32) -> Result<usize> {
        unsafe { (&mut *self.get()).poke(addr, val) }
    }

    fn poke_from_slice(&mut self, base: u32, data: &[u32]) -> Result<usize> {
        unsafe { (&mut *self.get()).poke_from_slice(base, data) }
    }

    fn dump(&self) -> Result<()> {
        unsafe { (&*self.get()).dump() }
    }

    fn print_stats(&self, recurse: bool) {
        unsafe { (&*self.get()).print_stats(recurse) }
    }
}
