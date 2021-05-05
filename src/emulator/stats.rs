///! Reportador de estatísticas de execução.

use std::time::Instant;

use crate::emulator::Instruction;

use color_eyre::eyre::{eyre, Result};

/// Um reportador de estatísticas de execução.
/// Armazena o total de ciclos gasto, bem como o tempo gasto
/// durante a execução.
pub struct StatsReporter {
    /// O total de instruções executadas, indexadas pelo tipo da 
    /// instrução.
    n_instructions: [usize; 5],
    /// O número de (mono)ciclos executados até agora.
    n_cycles: usize,
    /// O instante em que a execução começou.
    start: Option<Instant>,
    /// O instante em que a execução terminou.
    finish: Option<Instant>,
}

impl StatsReporter {
    /// Cria um novo `StatsReporter` zerado.
    pub fn new() -> StatsReporter {
        StatsReporter {
            n_instructions: [0; 5],
            n_cycles: 0,
            start: None,
            finish: None,
        }
    }

    /// Inicia a contagem de estatísticas e de tempo.
    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    /// Termina a contagem de estatísticas e de tempo.
    pub fn finish(&mut self) {
        self.finish = Some(Instant::now());
    }

    /// Adiciona uma instrução na contagem de instruções.
    pub fn add_instr(&mut self, instr: &Instruction) {
        self.n_instructions[instr.kind() as usize] += 1;
    }

    /// Adiciona o número de ciclos à contagem.
    pub fn add_cycles(&mut self, cycles: usize) {
        self.n_cycles += cycles;
    }

    /// Analisa as estatísticas e imprime os resultados na saída padrão.
    pub fn print_stats(&self) -> Result<()> {
        let start = *self
            .start
            .as_ref()
            .ok_or_else(|| eyre!("StatsReporter did not start!"))?;
        let finish = *self
            .finish
            .as_ref()
            .ok_or_else(|| eyre!("StatsReporter did not finish!"))?;

        let delta = finish - start;

        let total: usize = self.n_instructions.iter().sum();

        let ips = total as f64 / delta.as_secs_f64();

        println!();
        println!("Execution finished successfully!");
        println!("--------------------------");
        println!(
            "Instruction count: {} (R: {} I: {} J: {} FR: {} FI: {})",
            total,
            self.n_instructions[0],
            self.n_instructions[1],
            self.n_instructions[2],
            self.n_instructions[3],
            self.n_instructions[4],
        );
        println!("Simulation time: {:.2} sec", delta.as_secs_f32());
        println!("Average IPS: {:.2}", ips);

        println!();
        println!();

        println!("Simulated execution times for:");
        println!("---------------------------");

        println!("Monocycle");
        let mono_cycles = self.n_cycles;
        println!("\tCycles: {}", mono_cycles);
        let mono_freq = 33.8688 / 4.0;
        println!("\tFrequency: {:.4} MHz", mono_freq);
        let mono_time = (mono_cycles as f32) / mono_freq / 1_000_000.0;
        println!("\tEstimated execution time: {:.4} sec", mono_time);
        let mono_ipc = (total as f32) / (mono_cycles as f32);
        println!("\tIPC: {:.2}", mono_ipc);
        let mono_mips = mono_ipc * mono_freq;
        println!("\tMIPS: {:.2}", mono_mips);

        println!("Pipelined");
        let pipe_cycles = self.n_cycles + 4;
        println!("\tCycles: {}", pipe_cycles);
        let pipe_freq = 33.8688;
        println!("\tFrequency: {:.4} MHz", pipe_freq);
        let pipe_time = (pipe_cycles as f32) / pipe_freq / 1_000_000.0;
        println!("\tEstimated execution time: {:.4} sec", pipe_time);
        let pipe_ipc = (total as f32) / (pipe_cycles as f32);
        println!("\tIPC: {:.2}", pipe_ipc);
        let pipe_mips = pipe_ipc * pipe_freq;
        println!("\tMIPS: {:.2}", pipe_mips);

        let speedup = mono_time / pipe_time;
        println!("Speedup Monocycle/Pipeline: {:.2}x", speedup);

        Ok(())
    }
}
