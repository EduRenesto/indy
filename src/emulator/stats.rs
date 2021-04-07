use std::time::Instant;

use crate::emulator::Instruction;

use color_eyre::eyre::{ Result, eyre };

pub struct StatsReporter {
    n_instructions: [usize; 5],
    start: Option<Instant>,
    finish: Option<Instant>,
}

impl StatsReporter {
    pub fn new() -> StatsReporter {
        StatsReporter {
            n_instructions: [0; 5],
            start: None,
            finish: None,
        }
    }

    pub fn start(&mut self) {
        self.start = Some(Instant::now());
    }

    pub fn finish(&mut self) {
        self.finish = Some(Instant::now());
    }

    pub fn add_instr(&mut self, instr: &Instruction) {
        self.n_instructions[instr.kind() as usize] += 1;
    }

    pub fn print_stats(&self) -> Result<()> {
        let start = *self.start.as_ref().ok_or(eyre!("StatsReporter did not start!"))?;
        let finish = *self.finish.as_ref().ok_or(eyre!("StatsReporter did not finish!"))?;

        let delta = finish - start;

        let total: usize = self.n_instructions
            .iter()
            .sum();

        let ips = total as f64 / delta.as_secs_f64();

        println!("");
        println!("Execution finished successfully!");
        println!("--------------------------");
        println!("Instruction count: {} (R: {} I: {} J: {} FR: {} FI: {})",
            total,
            self.n_instructions[0],
            self.n_instructions[1],
            self.n_instructions[2],
            self.n_instructions[3],
            self.n_instructions[4],
        );
        println!("Simulation time: {:.2} sec", delta.as_secs_f32());
        println!("Average IPS: {:.2}", ips);

        Ok(())
    }
}
