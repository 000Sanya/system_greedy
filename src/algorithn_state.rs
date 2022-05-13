use bitvec::prelude::BitVec;
use crate::System;

pub struct AlgorithmState {
    pub minimal_state: State,
    pub steps: Vec<Step>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct State {
    pub state: BitVec,
    pub energy: f64,
    pub spin_excess: i32,
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
                spin_excess: 0
            },
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
            step_kind: step_type
        };
        if self.minimal_state.energy > system.energy() {
            self.minimal_state = step.state.clone();
        }
        self.steps.push(step);
    }
}
