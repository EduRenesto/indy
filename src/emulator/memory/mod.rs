use color_eyre::eyre::Result;

pub mod ram;
pub mod cache;

// TODO remover depois da integracao
pub use ram::Ram;

pub trait Memory {
    fn peek(&mut self, addr: u32) -> Result<u32>;
    fn poke(&mut self, addr: u32, val: u32) -> Result<()>;

    fn peek_unaligned(&mut self, addr: u32) -> Result<u8> {
        let base = addr & 0xFFFFFFFC; // alinha pro lowest multiplo de 4
        let offset = addr - base; // offset agora armazena qual é o byte desejado

        let word = self.peek(base)?;

        //Ok(((word & (0xFF << offset )) >> offset) as u8)
        Ok(word.to_le_bytes()[offset as usize])
    }
}
