use super::Memory;

use color_eyre::eyre::{ Result, eyre };

/// As políticas de substituição da cache.
#[derive(Copy, Clone, Debug)]
pub enum RepPolicy {
    /// Uma linha aleatória será escolhida para ser substituída.
    Random,
    /// A última linha utilizada será escolhida para ser substituída.
    LeastRecentlyUsed,
}

/// Uma linha de cache.
/// L é o tamanho em palavras da linha.
#[derive(Copy, Clone, Debug)]
struct Line<const L: usize> {
    /// A tag da linha.
    /// TODO trocar de u32 pra u8 e fazer o calculo direito!
    tag: u32,
    /// Verdadeiro se o conteúdo da linha atual pode não ser o mesmo
    /// que no próximo nível.
    dirty: bool,
    /// Os dados da linha.
    data: [u32; L],
}

/// Uma cache.
/// T é o tipo do próximo nível, L é o tamanho em palavras de cada linha,
/// N é o número de linhas e A é a associatividade da cache. Ou seja,
/// A=1 tem mapeamento direto, A=N é completamente associativa, etc etc.
pub struct Cache<T: Memory, const L: usize, const N: usize, const A: usize> {
    /// O próximo nível da hierarquia de memória.
    next: T,
    /// As linhas da cache.
    lines: [Option<Line<L>>; N],
    /// A política de substituicao das linhas.
    policy: RepPolicy,
    /// A latência de acesso do nível atual.
    latency: usize,
    /// O nome da cache, para debugging.
    name: &'static str,
}

/// Implementações comuns a todas as configurações de cache.
impl<T: Memory, const L: usize, const N: usize, const A: usize> Cache<T, L, N, A> {
    /// Cria uma nova cache.
    pub fn new(name: &'static str, next: T, policy: RepPolicy, latency: usize) -> Self {
        Cache {
            name,
            next,
            lines: [None; N],
            policy,
            latency,
        }
    }

    /// Se a flag `dirty` da linha ser verdadeira, então
    /// escreve o conteúdo no próximo nível. Senão, não faz nada.
    fn flush_line(&mut self, line_idx: usize) -> Result<()> {
        match &self.lines[line_idx] {
            Some(ref line) if line.dirty => {
                for i in 0..L {
                    self.next.poke(line.tag + i as u32, line.data[i])?;
                }

                Ok(())
            },
            _ => Ok(())
        }
    }

    /// Pega uma linha do próximo nível e o coloca na linha
    /// da cache. Ignora o conteúdo anterior da linha: tome cuidado!
    fn load_into_line(&mut self, line_idx: usize, base: u32) -> Result<()> {
        if let Some(line) = self.lines[line_idx].as_mut() {
            for i in 0..L {
                line.data[i] = self.next.peek(base + 4 * i as u32)?;
            }
            line.dirty = false;
        } else {
            let mut data = [0; L];
            for i in 0..L {
                data[i] = self.next.peek(base + 4 * i as u32)?;
            }

            self.lines[line_idx] = Some(Line {
                tag: base,
                dirty: false,
                data,
            });
        }

        Ok(())
    }
}

/// Implementação de uma cache com A=1, ou seja, com mapeamento direto.
impl<T: Memory, const L: usize, const N: usize> Memory for Cache<T, L, N, 1> {
    fn peek(&mut self, addr: u32) -> Result<u32> {
        let line_idx = addr as usize % (L * N);
        let offset = addr as usize % L;
        
        match self.lines[line_idx] {
            Some(ref line) if line.tag == (addr - offset as u32) => {
                // Hit!
                println!("cache {}: read access {:#010x} hit at line {:#010x} offset {:x}",
                         self.name, addr, line_idx, offset);

                return Ok(line.data[offset]);
            }, 
            _ => {
                // Miss!
                println!("cache {}: read access {:#010x} miss at line {:#010x} offset {:x}",
                         self.name, addr, line_idx, offset);

                // Como aqui é mapeamento direto, so existe uma linha possível de ser 
                // sobrescrita.

                // Faz o flush da linha antiga
                self.flush_line(line_idx)?;
                self.load_into_line(line_idx, addr - offset as u32)?;
            },
        }

        Ok(self.lines[line_idx].unwrap().data[offset])
    }

    fn poke(&mut self, addr: u32, val: u32) -> Result<()> {
        let line_idx = addr as usize % (L * N);
        let offset = addr as usize % L;

        match &mut self.lines[line_idx] {
            Some(ref mut line) if line.tag == (addr - offset as u32) => {
                println!("cache {}: write access {:#010x} hit at line {:#010x} offset {:x}",
                         self.name, addr, line_idx, offset);
                // A linha é nossa, só atualiza e seta o dirty
                line.data[offset] = val;
                line.dirty = true;
            }, 
            _ => {
                println!("cache {}: write access {:#010x} miss at line {:#010x} offset {:x}",
                         self.name, addr, line_idx, offset);
                // A linha não é nossa. Faz o flush e fetch do prox nível
                self.flush_line(line_idx)?;
                self.load_into_line(line_idx, addr - offset as u32)?;
            }
        }

        Ok(())
    }
}
