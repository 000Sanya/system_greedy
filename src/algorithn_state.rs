use crate::System;
use bitvec::prelude::BitVec;

pub struct AlgorithmState {
    pub minimal_state: State,
    pub new_minimal_state: bool,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub state: BitVec,
    pub energy: f64,
    pub spin_excess: i32,
}

impl Default for State {
    fn default() -> Self {
        Self {
            state: Default::default(),
            energy: f64::MAX,
            spin_excess: 0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct State2 {
    pub state: BitVec,
    pub energy: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    pub state: State,
    pub step_kind: StepKind,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum StepKind {
    Greedy,
    Step2,
    Step3,
    Minimize1,
    Minimize2,
}

impl AlgorithmState {
    pub fn new() -> Self {
        Self {
            minimal_state: State {
                state: Default::default(),
                energy: f64::MAX,
                spin_excess: 0,
            },
            new_minimal_state: false,
            steps: vec![],
        }
    }

    pub fn save_step_state(&mut self, system: &System, step_type: StepKind) {
        let step = Step {
            state: State {
                state: system.system_state().clone(),
                energy: system.energy(),
                spin_excess: system.spin_excess(),
            },
            step_kind: step_type,
        };
        if self.minimal_state.energy > system.energy() {
            self.minimal_state = step.state.clone();
            self.new_minimal_state = true;
        }
        self.steps.push(step);
    }

    pub fn save_step_state2(&mut self, system: &System, step_type: StepKind) {
        let step = Step {
            state: State {
                state: system.system_state().clone(),
                energy: system.energy(),
                spin_excess: system.spin_excess(),
            },
            step_kind: step_type,
        };
        if self.minimal_state.energy > system.energy() {
            self.minimal_state = step.state;
            self.new_minimal_state = true;
            // system.save_system(format!("results/min_{}.mfsys", system.energy()));
        }
    }

    pub fn clear_steps(&mut self) {
        self.steps.clear();
    }

    pub fn consume_minimal_state(&mut self) -> Option<State> {
        if self.new_minimal_state {
            self.new_minimal_state = false;
            Some(self.minimal_state.clone())
        } else {
            None
        }
    }
}
