use super::Memory;

use std::cell::UnsafeCell;

use log::debug;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
    thread_rng,
};

use color_eyre::eyre::Result;

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
#[derive(Copy, Clone)]
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

impl<const L: usize> std::fmt::Debug for Line<L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Line {{ tag: {:#010x}, dirty: {:?}, data: [ ",
            self.tag, self.dirty
        )?;
        for d in self.data.iter() {
            write!(f, "{:#010x} ", d)?;
        }
        write!(f, "]}}")?;

        Ok(())
    }
}

enum FindLine {
    Hit(usize, usize),
    Miss(usize, usize),
}

/// Uma cache.
/// T é o tipo do próximo nível, L é o tamanho em palavras de cada linha,
/// N é o número de linhas e A é a associatividade da cache. Ou seja,
/// A=1 tem mapeamento direto, A=N é completamente associativa, etc etc.
pub struct Cache<'a, T: Memory, const L: usize, const N: usize, const A: usize> {
    /// O próximo nível da hierarquia de memória.
    next: &'a UnsafeCell<T>,
    /// As linhas da cache.
    lines: [Option<Line<L>>; N],
    /// A política de substituicao das linhas.
    policy: RepPolicy,
    /// A latência de acesso do nível atual.
    latency: usize,
    /// O nome da cache, para debugging.
    name: &'static str,
    /// A quantidade de acessos.
    accesses: usize,
    /// A quantidade de misses.
    misses: usize,
    /// Um gerador aleatorio para o line replacing.
    rng: ThreadRng,
}

impl<'a, T: Memory, const L: usize, const N: usize, const A: usize> Drop for Cache<'a, T, L, N, A> {
    fn drop(&mut self) {
        let hits = self.accesses - self.misses;
        let hit_rate = (hits as f32) / (self.accesses as f32) * 100.0;
        let miss_rate = (self.misses as f32) / (self.accesses as f32) * 100.0;
        println!(
            "Cache {}: {} accesses, {} ({:.2}%) hits, {} ({:.2}%) misses",
            self.name, self.accesses, hits, hit_rate, self.misses, miss_rate
        );
    }
}

/// Implementações comuns a todas as configurações de cache.
impl<'a, T: Memory, const L: usize, const N: usize, const A: usize> Cache<'a, T, L, N, A> {
    /// Cria uma nova cache.
    pub fn new(
        name: &'static str,
        next: &'a UnsafeCell<T>,
        policy: RepPolicy,
        latency: usize,
    ) -> Self {
        Cache {
            name,
            next,
            lines: [None; N],
            policy,
            latency,
            accesses: 0,
            misses: 0,
            rng: thread_rng(),
        }
    }

    /// Acha a linha em que o endereço está.
    fn find_line(&mut self, addr: u32) -> FindLine {
        //let lines_per_block = N / A;
        let lines_per_block = A;
        let n_blocks = N / A;

        let offset = (addr / 4) as usize % L;
        let base = addr - (4 * offset as u32);

        let block_idx = (base / 4) as usize % n_blocks;

        for i in 0..lines_per_block {
            let line_idx = block_idx * lines_per_block + i;
            match &self.lines[line_idx] {
                Some(ref line) if line.tag == base => {
                    return FindLine::Hit(line_idx, offset);
                }
                _ => continue,
            }
        }

        if A == 1 {
            let line_idx = (base / 4) as usize % N;
            return FindLine::Miss(line_idx, offset);
        }

        match self.policy {
            RepPolicy::Random => {
                // TODO da pra tirar umas coisas daq
                let dist = Uniform::new(0, lines_per_block);
                let line_idx = block_idx * lines_per_block + dist.sample(&mut self.rng);

                FindLine::Miss(line_idx, offset)
            }
            RepPolicy::LeastRecentlyUsed => {
                todo!()
            }
        }
    }

    /// Se a flag `dirty` da linha ser verdadeira, então
    /// escreve o conteúdo no próximo nível. Senão, não faz nada.
    fn flush_line(&mut self, line_idx: usize) -> Result<()> {
        match &self.lines[line_idx] {
            Some(ref line) if line.dirty => {
                debug!(
                    "cache {}: flushing line {:#010x} to {:#010x}",
                    self.name, line_idx, line.tag
                );
                unsafe {
                    for i in 0..L {
                        (&mut *self.next.get()).poke(line.tag + 4 * i as u32, line.data[i])?;
                    }
                }

                Ok(())
            }
            _ => Ok(()),
        }
    }

    /// Pega uma linha do próximo nível e o coloca na linha
    /// da cache. Ignora o conteúdo anterior da linha: tome cuidado!
    fn load_into_line(&mut self, line_idx: usize, base: u32) -> Result<()> {
        debug!(
            "cache {}: loading {:#010x} to line {:#010x}",
            self.name, base, line_idx
        );
        if let Some(line) = self.lines[line_idx].as_mut() {
            unsafe {
                for i in 0..L {
                    line.data[i] = (&mut *self.next.get()).peek(base + 4 * i as u32)?;
                }
            }
            line.dirty = false;
            line.tag = base;
        } else {
            let mut data = [0; L];
            unsafe {
                for i in 0..L {
                    data[i] = (&mut *self.next.get()).peek(base + 4 * i as u32)?;
                }
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

impl<'a, T: Memory, const L: usize, const N: usize, const A: usize> Memory
    for Cache<'a, T, L, N, A>
{
    fn peek(&mut self, addr: u32) -> Result<u32> {
        self.accesses += 1;

        let offset = (addr / 4) as usize % L;
        let base = addr - (4 * offset as u32);

        match self.find_line(addr) {
            FindLine::Hit(line_idx, offset) => {
                debug!(
                    "cache {}: read access {:#010x} hit at line {:#010x} offset {:x}",
                    self.name, addr, line_idx, offset
                );

                let line = self.lines[line_idx].unwrap();
                Ok(line.data[offset])
            }
            FindLine::Miss(line_idx, offset) => {
                self.misses += 1;
                debug!(
                    "cache {}: read access {:#010x} miss at line {:#010x} offset {:x}",
                    self.name, addr, line_idx, offset
                );

                // Faz o flush da linha antiga
                self.flush_line(line_idx)?;
                self.load_into_line(line_idx, base)?;

                let line = self.lines[line_idx].unwrap();

                Ok(line.data[offset])
            }
        }
    }

    fn poke(&mut self, addr: u32, val: u32) -> Result<()> {
        self.accesses += 1;
        let offset = (addr / 4) as usize % L;
        let base = addr - (4 * offset as u32);

        match self.find_line(addr) {
            FindLine::Hit(line_idx, offset) => {
                debug!(
                    "cache {}: read access {:#010x} hit at line {:#010x} offset {:x}",
                    self.name, addr, line_idx, offset
                );

                let mut line = self.lines[line_idx].as_mut().unwrap();
                line.data[offset] = val;
                line.dirty = true;
            }
            FindLine::Miss(line_idx, offset) => {
                self.misses += 1;
                debug!(
                    "cache {}: read access {:#010x} miss at line {:#010x} offset {:x}",
                    self.name, addr, line_idx, offset
                );

                // Faz o flush da linha antiga
                self.flush_line(line_idx)?;
                self.load_into_line(line_idx, base)?;

                let mut line = self.lines[line_idx].as_mut().unwrap();
                line.data[offset] = val;
                line.dirty = true;
            }
        }

        Ok(())
    }

    fn dump(&self) -> Result<()> {
        println!("===== Dump of cache {} =====", self.name);

        for (idx, line) in self.lines.iter().enumerate() {
            println!("line {:#010x}: {:?}", idx, line);
        }

        println!("============================");

        Ok(())
    }
}
