use system_greedy::generators::LatticeGenerator;
use system_greedy::metropolis::metropolis_mc_step;
use system_greedy::runner::{AlgorithmState, Replicate, runner_multi_thread, StateRegisterer};
use system_greedy::system::System;

pub struct MetropolisState {
    temp: f64,
    step: f64,
    min_temp: f64,
}

impl MetropolisState {
    pub fn new(temp: f64, step: f64, min_temp: f64) -> Self {
        Self { temp, step, min_temp }
    }
}

impl AlgorithmState for MetropolisState {
    fn after_step(&mut self) {
        self.temp = self.min_temp.max(self.temp * self.step);
    }

    fn after_step_for_system<SR: StateRegisterer>(&self, system: &mut System, registerer: &SR) {
        Replicate.after_step_for_system(system, registerer);
    }
}

fn main() {
    let start_temp = 10000000.0f64;
    let end_temp = 100f64;
    let mc_steps = 10000.;
    let step = (end_temp / start_temp).powf(1. / mc_steps);

    let mut system = LatticeGenerator::trimer(450.0 / 2.0, 700., 4, 3);
    let mut state = MetropolisState::new(1.0, step, end_temp);
    runner_multi_thread(system, "trim", state, mc_steps as usize, 6, |system, registerer, state| {
        metropolis_mc_step(system, registerer, state.temp, system.size() * 10000);
    });
}