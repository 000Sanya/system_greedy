use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, Ordering};
use bitvec::vec::BitVec;
use num_traits::Zero;
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use crate::System;

#[derive(Debug, Clone)]
pub struct State {
    pub energy: f64,
    pub state: BitVec,
}

impl Default for State {
    fn default() -> Self {
        Self {
            energy: f64::MAX,
            state: Default::default(),
        }
    }
}

pub struct StateRegistererInner {
    current_minimal: Option<State>,
    previous_minimal: Option<State>,
    is_changed: bool,
}

impl StateRegistererInner {
    pub fn new() -> Self {
        Self {
            current_minimal: None,
            previous_minimal: None,
            is_changed: false,
        }
    }

    pub fn register(&mut self, system: &System) {
        if self.current_minimal.as_ref().map_or(f64::MAX, |x| x.energy) > system.energy() {
            self.previous_minimal = self.current_minimal.replace(State {
                energy: system.energy(),
                state: system.system_state().clone(),
            });
            self.is_changed = true;
        }
    }

    pub fn minimal_state(&self) -> Option<State> {
        self.current_minimal.clone()
    }

    pub fn check_if_changed(&mut self) -> bool {
        std::mem::replace(&mut self.is_changed, false)
    }

    pub fn diff_between_mins(&self, eps: f64) -> bool {
        self.current_minimal
            .as_ref()
            .map(|x| x.energy)
            .zip(self.previous_minimal.as_ref().map(|x| x.energy))
            .map(|(e1, e2)| (e1 - e2).abs() < eps)
            .unwrap_or(false)
    }
}

pub trait StateRegisterer {
    fn register(&self, system: &System);
    fn minimal_state(&self) -> Option<State>;
}

pub struct RefCellStateRegisterer(pub RefCell<StateRegistererInner>);

impl StateRegisterer for RefCellStateRegisterer {
    fn register(&self, system: &System) {
        self.0.borrow_mut().register(system);
    }

    fn minimal_state(&self) -> Option<State> {
        self.0.borrow().minimal_state()
    }
}

pub struct MutexStateRegisterer(pub Mutex<StateRegistererInner>);

impl StateRegisterer for MutexStateRegisterer {
    fn register(&self, system: &System) {
        self.0.lock().unwrap().register(system);
    }

    fn minimal_state(&self) -> Option<State> {
        self.0.lock().unwrap().minimal_state()
    }
}

pub trait AlgorithmState {
    fn after_step(&mut self);

    fn after_step_for_system<SR: StateRegisterer>(&self, system: &mut System, registerer: &SR);
}

impl AlgorithmState for () {
    #[inline(always)]
    fn after_step(&mut self) {}

    #[inline(always)]
    fn after_step_for_system<SR: StateRegisterer>(&self, _system: &mut System, _registerer: &SR) {}
}

pub struct Replicate;

impl AlgorithmState for Replicate {
    #[inline(always)]
    fn after_step(&mut self) {}

    fn after_step_for_system<SR: StateRegisterer>(&self, system: &mut System, registerer: &SR) {
        if let Some(state) = registerer.minimal_state() {
            system.set_system_state(state.state);
        }
    }
}

pub fn runner_one_thread<S: AlgorithmState, F: FnMut(&mut System, &RefCellStateRegisterer, &S)>(
    mut system: System,
    max_steps: usize,
    mut algorithm_state: S,
    mut f: F,
) -> State {
    let mut state_register = RefCellStateRegisterer(RefCell::new(StateRegistererInner::new()));
    let mut steps = 0;

    while !state_register.0.borrow().diff_between_mins(1e-8) {
        if steps >= max_steps {
            break;
        }

        f(&mut system, &state_register, &algorithm_state);

        algorithm_state.after_step();
        algorithm_state.after_step_for_system(&mut system, &state_register);

        if state_register.0.borrow_mut().check_if_changed() {
            steps = 0;
        } else {
            steps += 1;
        }
    }

    state_register.minimal_state().unwrap()
}

lazy_static::lazy_static! {
    static ref WORKING: Arc<AtomicBool> = {
        let working = Arc::new(AtomicBool::new(true));
        let w = working.clone();
        ctrlc::set_handler(move || {
            println!("Wait step for closing");
            w.store(false, Ordering::SeqCst);
        }).unwrap();
        working
    };
}

pub fn runner_multi_thread<S: AlgorithmState + Sync, F: Fn(&mut System, &MutexStateRegisterer, &S) + Sync + Send>(
    mut system: System,
    mut algorithm_state: S,
    max_steps: usize,
    thread_count: usize,
    mut f: F,
) -> State {
    let mut state_register = MutexStateRegisterer(Mutex::new(StateRegistererInner::new()));
    let mut steps = 0;
    let mut all_steps = 0;

    let mut systems = vec![system.clone(); thread_count];
    while !state_register.0.lock().unwrap().diff_between_mins(1e-8) && WORKING.load(Ordering::SeqCst) {

        if steps >= max_steps {
            break;
        }

        systems = systems
            .into_par_iter()
            .map(|mut system| {
                f(&mut system, &state_register, &algorithm_state);
                system
            })
            .collect();

        algorithm_state.after_step();
        for system in &mut systems {
            algorithm_state.after_step_for_system(system, &state_register);
        }

        if state_register.0.lock().unwrap().check_if_changed() {
            steps = 0;
        } else {
            steps += 1;
        }

        all_steps += 1;
    }

    state_register.minimal_state().unwrap()
}

