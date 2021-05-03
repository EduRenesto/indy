use super::reporter::MemoryEvent;
use super::{Memory, MemoryStats};

use std::cell::UnsafeCell;
use std::sync::mpsc::Sender;

use log::debug;
use rand::{
    distributions::{Distribution, Uniform},
    rngs::ThreadRng,
    thread_rng,
};

use color_eyre::eyre::Result;

/// Calcula o log2 de um inteiro em O(n)
const fn log2_iter(n: usize) -> usize {
    let mut n = n;
    let mut i = 0;

    while n != 1 {
        n = n >> 1;
        i += 1;
    }

    i
}

/// Calcula o log2 de um inteiro usando uma lookup table.
/// Se n \in [1; 1024], retorna em O(1).
/// Na real, é um quick hack. :'B
const fn log2_lut(n: usize) -> usize {
    match n.next_power_of_two() {
        1 => 0,
        2 => 1,
        4 => 2,
        8 => 3,
        16 => 4,
        32 => 5,
        64 => 6,
        128 => 7,
        256 => 8,
        512 => 9,
        1024 => 10,
        n => log2_iter(n),
    }
}

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
    tag: usize,
    /// Verdadeiro se o conteúdo da linha atual pode não ser o mesmo
    /// que no próximo nível.
    dirty: bool,
    /// Verdadeiro se o conteúdo da linha atual é consistente com o resto
    /// da hierarquia de memória.
    valid: bool,
    /// O "número" do último acesso a esta linha.
    last_access: usize,
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

#[derive(Copy, Clone)]
struct LineIndex {
    /// O número de linha.
    line_number: usize,
    /// O índice do set.
    set_idx: usize,
    /// O offset da palavra dentro da linha.
    offset: usize,
    /// O índice da linha dentro da array `lines`.
    line_idx: usize,
    /// A tag da linha.
    tag: usize,
}

impl LineIndex {
    pub fn to_addr<const L: usize>(&self) -> u32 {
        //let set_size = A;
        //let n_sets = N / set_size;
        //let n_sets_bits = log2_lut(n_sets);

        //((self.line_number << n_sets_bits) | self.set_idx) as u32

        (self.line_number * L * 4) as u32
    }
}

enum FindLine {
    Hit(LineIndex),
    Miss(LineIndex),
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
    ///  nome da cache, para debugging.
    name: &'static str,
    /// A quantidade de acessos.
    accesses: usize,
    /// A quantidade de misses.
    misses: usize,
    /// Um gerador aleatorio para o line replacing.
    rng: ThreadRng,
    /// O write-end de um Memory Reporter.
    reporter: Option<Sender<MemoryEvent>>,
    /// A cache irmã, se existente.
    sister: Option<&'a UnsafeCell<Cache<'a, T, L, N, A>>>,
    /// `true` se a cache deve procurar na irmã antes de ir ao próximo nível.
    fetch_from_sister: bool,
}

/// Implementações comuns a todas as configurações de cache.
impl<'a, T: Memory, const L: usize, const N: usize, const A: usize> Cache<'a, T, L, N, A> {
    /// Cria uma nova cache.
    pub fn new(
        name: &'static str,
        next: &'a UnsafeCell<T>,
        policy: RepPolicy,
        latency: usize,
        reporter: Option<Sender<MemoryEvent>>,
    ) -> Self {
        let set_size = A;
        let n_sets = N / set_size;
        let n_sets_bits = log2_lut(n_sets);

        debug!("cache {}: set_size = {}", name, set_size);
        debug!("cache {}: n_sets = {}", name, n_sets);
        debug!("cache {}: n_sets_bits {}", name, n_sets_bits);

        Cache {
            name,
            next,
            lines: [None; N],
            policy,
            latency,
            accesses: 0,
            misses: 0,
            rng: thread_rng(),
            reporter,
            sister: None,
            fetch_from_sister: false,
        }
    }

    /// Define `sister` como a cache irmã.
    pub fn set_sister(&mut self, sister: &'a UnsafeCell<Cache<'a, T, L, N, A>>, fetch: bool) {
        self.sister.replace(sister);
        self.fetch_from_sister = fetch;
    }

    /// Calcula só a tag de um endereço.
    fn calc_tag(&self, addr: u32) -> usize {
        let set_size = A;
        let n_sets = N / set_size;
        let n_sets_bits = log2_lut(n_sets);

        let line_number = addr as usize / (L * 32);
        let tag = line_number >> n_sets_bits;

        tag
    }

    /// Acha a linha em que o endereço está.
    fn find_line(&mut self, addr: u32) -> FindLine {
        //let set_size = (32 * L * A);
        //let n_sets = (N * L * 32) / set_size;
        let set_size = A;
        let n_sets = N / set_size;
        let n_sets_bits = log2_lut(n_sets);

        //debug!(
        //    "cache {}: calculating tag for address {:#010x}",
        //    self.name, addr
        //);
        //debug!("cache {}: |=> addr        = {:#034b}", self.name, addr);
        let line_number = addr as usize / (L * 4);
        //debug!(
        //    "cache {}: |=> line_number = {:#034b}",
        //    self.name, line_number
        //);
        let set_idx = line_number & ((n_sets).next_power_of_two() - 1); // Isso só funciona com potências de 2!!!!!
        //debug!("cache {}: |=> set_idx     = {:#034b}", self.name, set_idx);
        let tag = line_number >> n_sets_bits;
        //debug!("cache {}: |=> tag         = {:#034b}", self.name, tag);

        debug!(
            "cache {}: addr {:#010x} => line {:#010x}, tag {:#010x}",
            self.name, addr, line_number, tag
        );

        for i in 0..set_size {
            let line_idx = set_idx * set_size + i;
            match &self.lines[line_idx] {
                Some(ref line) if line.tag == tag => {
                    let offset = (addr as usize / 4) % L;
                    debug!("cache {}: found at way {}", self.name, line_idx);
                    self.print_to_debug(format!("{}: {:#010x} tag matched at line {:#010x} way {}", 
                                        self.name, addr, line_number, i));
                    return FindLine::Hit(LineIndex {
                        line_number,
                        set_idx,
                        offset,
                        line_idx,
                        tag,
                    });
                }
                _ => continue,
            }
        }

        self.print_to_debug(format!("{}: miss", self.name));

        //if A == 1 {
        //    let line_idx = (base / 4) as usize % N;
        //    return FindLine::Miss(line_idx, offset);
        //}

        match self.policy {
            RepPolicy::Random => {
                // TODO da pra tirar umas coisas daq
                let dist = Uniform::new(0, set_size);
                let way = dist.sample(&mut self.rng);
                let line_idx = set_idx * set_size + way;

                debug!("cache {}: randomly replacing way {}", self.name, line_idx);

                self.print_to_debug(format!("\trandomly choosing way {}", way));

                let offset = (addr as usize / 4) % L;
                FindLine::Miss(LineIndex {
                    line_number,
                    set_idx,
                    offset,
                    line_idx,
                    tag,
                })
            }
            RepPolicy::LeastRecentlyUsed => {
                let (_, idx, way) = (0..set_size)
                    .into_iter()
                    .map(|i| {
                        let line_idx = set_idx * set_size + i;

                        match self.lines[line_idx] {
                            Some(ref line) => { 
                                debug!("cache {}: age of line {:#010x} way {}: {}",
                                       self.name, line_number, i, line.last_access);
                                (line.last_access, line_idx, i)
                            },
                            None => { 
                                debug!("cache {}: age of line {:#010x} way {}: 0",
                                       self.name, line_number, i);
                                (0, line_idx, i) 
                            },
                        }
                    })
                    .min()
                    .unwrap();

                debug!("cache {}: LRU choose way {}", self.name, way);

                self.print_to_debug(format!("\tLRU-choosing way {}", way));

                let offset = (addr as usize / 4) % L;
                FindLine::Miss(LineIndex {
                    line_number,
                    set_idx,
                    offset,
                    line_idx: idx,
                    tag,
                })
            }
        }
    }

    /// Se a flag `dirty` da linha ser verdadeira, então
    /// escreve o conteúdo no próximo nível. Senão, não faz nada.
    /// Retorna o total de ciclos gastos.
    fn flush_line(&mut self, idx: &LineIndex) -> Result<usize> {
        match &self.lines[idx.line_idx] {
            Some(ref line) if line.dirty => {
                let set_size = A;
                let n_sets = N / set_size;
                let n_sets_bits = log2_lut(n_sets);

                let old_line_no = (line.tag << n_sets_bits) | idx.set_idx;
                let base = (4 * L * old_line_no) as u32;

                debug!(
                    "cache {}: flushing line {:#010x} ({}) to {:#010x}",
                    self.name, idx.line_number, idx.line_idx, base
                );

                self.print_to_debug(format!("\tflushing line {:#010x}", idx.line_number));

                if let Some(sister) = self.sister {
                    unsafe {
                        (&mut *sister.get()).invalidate_line(idx.to_addr::<L>()); // TODO usar line_number
                    }
                }

                let cycles = unsafe {
                    let mut total_cycles = 0;
                    //let addr = idx.to_addr::<L>();
                    for i in 0..L {
                        let a = base + 4 * i as u32;
                        debug!(
                            "cache {}: next[{:#010x}] <- {:#010x}[{}]",
                            self.name, a, idx.line_number, i
                        );
                        let cycles = (&mut *self.next.get()).poke(a, line.data[i])?;

                        if cycles > total_cycles {
                            total_cycles = cycles;
                        }
                    }

                    total_cycles
                };

                Ok(cycles)
            }
            _ => {
                debug!(
                    "cache {}: no need to write back line {:#010x}",
                    self.name, idx.line_number
                );
                self.print_to_debug(format!("\tno need to write back line {:#010x}", idx.line_number));
                Ok(0)
            }
        }
    }

    /// Pega uma linha do próximo nível e o coloca na linha
    /// da cache. Ignora o conteúdo anterior da linha: tome cuidado!
    /// Retorna o total de ciclos gasto.
    fn load_into_line(&mut self, idx: &LineIndex, base: u32) -> Result<usize> {
        let mut total_cycles = 0;

        debug!(
            "cache {}: loading {:#010x} to line {:#010x} ({})",
            self.name, base, idx.line_number, idx.line_idx,
        );

        self.print_to_debug(format!("\tloading line {:#010x} from {:#010x}", idx.line_number, base));

        //let tag = self.calc_tag(base);

        if let Some(line) = self.lines[idx.line_idx].as_mut() {
            unsafe {
                for i in 0..L {
                    let a = base + 4 * i as u32;
                    let (d, cycles) = (&mut *self.next.get()).peek(a)?;
                    debug!(
                        "cache {}: {:#010x}[{}] <- next[{:#010x}] ({:#010x})",
                        self.name, idx.line_number, i, a, d
                    );
                    line.data[i] = d;
                    if cycles > total_cycles {
                        total_cycles = cycles;
                    }
                }
            }
            line.dirty = false;
            line.tag = idx.tag;
            line.valid = true;
        } else {
            let mut data = [0; L];
            unsafe {
                for i in 0..L {
                    let a = base + 4 * i as u32;
                    let (d, cycles) = (&mut *self.next.get()).peek(a)?;
                    debug!(
                        "cache {}: {:#010x}[{}] <- next[{:#010x}] ({:#010x})",
                        self.name, idx.line_number, i, a, d
                    );
                    data[i] = d;
                    if cycles > total_cycles {
                        total_cycles = cycles;
                    }
                }
            }

            self.lines[idx.line_idx] = Some(Line {
                tag: idx.tag,
                dirty: false,
                data,
                valid: true,
                last_access: self.accesses,
            });
        }

        Ok(total_cycles)
    }

    /// Lógica compartilhada
    fn do_peek(&mut self, addr: u32) -> Result<(LineIndex, u32, usize)> {
        self.accesses += 1;

        let offset = (addr / 4) as usize % L;
        let base = addr - (4 * offset as u32);

        match self.find_line(addr) {
            FindLine::Hit(idx) if self.lines[idx.line_idx].unwrap().valid => {
                // Hit válido
                self.print_to_debug("\tline is valid: hit!".to_string());
                debug!(
                    "cache {}: read access {:#010x} hit at line {:#010x} ({}) offset {:x}",
                    self.name, addr, idx.line_number, idx.line_idx, idx.offset
                );

                let line = self.lines[idx.line_idx].as_mut().unwrap();
                line.last_access = self.accesses;
                Ok((idx, line.data[idx.offset], self.latency))
            }
            FindLine::Hit(idx) => {
                // Hit inválido
                // i.e. registra miss e tenta pegar da irmã.
                // se não rolar na irmã, aí é miss de verdade.
                // DISCLAIMER: aqui não é preciso checar se tá dirty ou não
                // antes de fazer o load.
                // Só as caches de instruções vão cair nesse ramo, e não há escrita a partir
                // delas. Então, os bits de dirty sempre vão ser false.

                self.misses += 1;
                debug!(
                    "cache {}: read access {:#010x} invalid hit at line {:#010x} ({}) offset {:x}",
                    self.name, addr, idx.line_number, idx.line_idx, idx.offset
                );
                self.print_to_debug("\tline is invalid: miss!".to_string());

                if self.try_copy_from_sister(&idx) {
                    self.print_to_debug(format!("\tfound in sister, copying"));
                    debug!("cache {}: line {:#010x} found in sister, copying...", self.name,
                           idx.line_number);
                    let mut line = self.lines[idx.line_idx].as_mut().unwrap();
                    line.last_access = self.accesses;
                    Ok((idx, line.data[idx.offset], self.latency))
                } else {
                    self.print_to_debug(format!("\tnot found in sister, querying next level"));
                    debug!(
                        "cache {}: line {:#010x} not found in sister, querying next level...",
                        self.name, idx.line_number,
                    );

                    let mut cycles = 0;

                    // Faz o flush da linha antiga
                    cycles += self.flush_line(&idx)?;
                    cycles += self.load_into_line(&idx, base)?;

                    let mut line = self.lines[idx.line_idx].as_mut().unwrap();
                    line.last_access = self.accesses;

                    Ok((idx, line.data[idx.offset], cycles + self.latency))
                }
            }
            FindLine::Miss(idx) => {
                self.misses += 1;
                debug!(
                    "cache {}: read access {:#010x} miss at line {:#010x} ({}) offset {:x}",
                    self.name, addr, idx.line_number, idx.line_idx, idx.offset
                );

                if self.try_copy_from_sister(&idx) {
                    self.print_to_debug(format!("\tfound in sister, copying"));
                    debug!("cache {}: line {:#010x} found in sister, copying...", self.name,
                           idx.line_number);
                    let mut line = self.lines[idx.line_idx].as_mut().unwrap();
                    line.last_access = self.accesses;
                    Ok((idx, line.data[idx.offset], self.latency))
                } else {
                    self.print_to_debug(format!("\tnot found in sister, querying next level"));
                    debug!(
                        "cache {}: line {:#010x} not found in sister, querying next level...",
                        self.name, idx.line_number,
                    );

                    let mut cycles = 0;

                    // Faz o flush da linha antiga
                    cycles += self.flush_line(&idx)?;
                    cycles += self.load_into_line(&idx, base)?;

                    let mut line = self.lines[idx.line_idx].as_mut().unwrap();
                    line.last_access = self.accesses;

                    Ok((idx, line.data[idx.offset], cycles + self.latency))
                }

                //let mut cycles = 0;

                //// Faz o flush da linha antiga
                //cycles += self.flush_line(&idx)?;
                //cycles += self.load_into_line(&idx, base)?;

                //let line = self.lines[idx.line_idx].unwrap();

                //Ok((idx, line.data[idx.offset], cycles + self.latency))
            }
        }
    }

    /// Invalida a linha da cache que contém esse endereço.
    fn invalidate_line(&mut self, addr: u32) {
        if let FindLine::Hit(idx) = self.find_line(addr) {
            debug!("cache {}: invalidating line {:#010x}", self.name, idx.line_number);
            self.lines[idx.line_idx].as_mut().unwrap().valid = false;
        }
    }

    /// Tenta copiar uma linha da cache irmã, se existente.
    /// Retorna `true` se foi possível.
    fn try_copy_from_sister(&mut self, idx: &LineIndex) -> bool {
        if self.sister.is_none() || !self.fetch_from_sister {
            return false;
        }

        let sister = unsafe {
            &*self.sister.unwrap().get()
        };

        for i in 0..A {
            let line_idx = idx.set_idx * A + i;
            match &sister.lines[line_idx] {
                Some(ref line) if line.tag == idx.tag => {
                    // Achou na irmã!
                    self.lines[idx.line_idx].replace(*line);
                    return true;
                }
                _ => continue,
            }
        }

        false
    }

    fn print_to_debug(&self, text: String) {
        if let Some(tx) = &self.reporter {
            tx.send(MemoryEvent::Debug(text)).unwrap();
        }
    }
}

impl<'a, T: Memory, const L: usize, const N: usize, const A: usize> Memory
    for Cache<'a, T, L, N, A>
{
    fn peek(&mut self, addr: u32) -> Result<(u32, usize)> {
        let (line_idx, data, cycles) = self.do_peek(addr)?;

        if let Some(ref tx) = self.reporter {
            tx.send(MemoryEvent::DataRead(addr, line_idx.line_number))?;
            tx.send(MemoryEvent::Debug(format!("====================")))?;
        }

        Ok((data, cycles))
    }

    fn peek_instruction(&mut self, addr: u32) -> Result<(u32, usize)> {
        let (line_idx, data, cycles) = self.do_peek(addr)?;

        if let Some(ref tx) = self.reporter {
            tx.send(MemoryEvent::InstrRead(addr, line_idx.line_number))?;
            tx.send(MemoryEvent::Debug(format!("====================")))?;
        }

        Ok((data, cycles))
    }

    fn poke(&mut self, addr: u32, val: u32) -> Result<usize> {
        self.accesses += 1;

        if let Some(sister) = self.sister {
            unsafe {
                // Se a irmã tem esse endereço em uma linha, então a invalide.
                (&mut *sister.get()).invalidate_line(addr);
            }
        }

        match self.find_line(addr) {
            FindLine::Hit(idx) => {
                debug!(
                    "cache {}: write access {:#010x} hit at line {:#010x} ({}) offset {:x}",
                    self.name, addr, idx.line_number, idx.line_idx, idx.offset
                );

                let mut line = self.lines[idx.line_idx].as_mut().unwrap();
                line.data[idx.offset] = val;
                line.dirty = true;
                line.last_access = self.accesses;

                debug!(
                    "cache {}: line {:#010x} ({}) marked dirty",
                    self.name, idx.line_number, idx.line_idx
                );

                if let Some(ref tx) = self.reporter {
                    tx.send(MemoryEvent::Write(addr, idx.line_number))?;
                    tx.send(MemoryEvent::Debug(format!("====================")))?;
                }

                Ok(self.latency)
            }
            FindLine::Miss(idx) => {
                self.misses += 1;
                debug!(
                    "cache {}: write access {:#010x} miss at line {:#010x} ({}) offset {:x}",
                    self.name, addr, idx.line_number, idx.line_idx, idx.offset
                );

                let offset = (addr / 4) as usize % L;
                let base = addr - (4 * offset as u32);

                let mut cycles = 0;

                // Faz o flush da linha antiga
                cycles += self.flush_line(&idx)?;
                cycles += self.load_into_line(&idx, base)?;

                let mut line = self.lines[idx.line_idx].as_mut().unwrap();
                line.data[idx.offset] = val;
                line.dirty = true;
                line.last_access = self.accesses;

                if let Some(ref tx) = self.reporter {
                    tx.send(MemoryEvent::Write(addr, idx.line_number))?;
                    tx.send(MemoryEvent::Debug(format!("====================")))?;
                }

                Ok(cycles + self.latency)
            }
        }
    }

    fn dump(&self) -> Result<()> {
        println!("===== Dump of cache {} =====", self.name);

        for (idx, line) in self.lines.iter().enumerate() {
            println!("line {:#010x}: {:?}", idx, line);
        }

        println!("============================");

        Ok(())
    }

    fn print_stats(&self, recurse: bool) {
        let hits = self.accesses - self.misses;
        let miss_rate = (self.misses as f32) / (self.accesses as f32);

        println!(
            "{:>5}  {:>12}  {:>12}  {:>12}   {:>8.2}%",
            self.name, hits, self.misses, self.accesses, miss_rate * 100.0,
        );

        if recurse {
            self.next.print_stats(true);
        }
    }
}
