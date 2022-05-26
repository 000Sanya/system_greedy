use std::cell::RefCell;
use std::sync::Mutex;
use bitvec::vec::BitVec;
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
            dbg!(system.energy());
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

pub struct RefCellStateRegisterer(RefCell<StateRegistererInner>);

impl StateRegisterer for RefCellStateRegisterer {
    fn register(&self, system: &System) {
        self.0.borrow_mut().register(system);
    }

    fn minimal_state(&self) -> Option<State> {
        self.0.borrow().minimal_state()
    }
}

pub struct MutexStateRegisterer(Mutex<StateRegistererInner>);

impl StateRegisterer for MutexStateRegisterer {
    fn register(&self, system: &System) {
        self.0.lock().unwrap().register(system);
    }

    fn minimal_state(&self) -> Option<State> {
        self.0.lock().unwrap().minimal_state()
    }
}

pub fn runner_one_thread<F: FnMut(&mut System, &RefCellStateRegisterer)>(mut system: System, lattice_name: &str, mut f: F) {
    let mut state_register = RefCellStateRegisterer(RefCell::new(StateRegistererInner::new()));
    let mut steps = 0;

    measure_time::print_time!("All");
    while !state_register.0.borrow().diff_between_mins(1e-8) || steps < 10 {
        measure_time::print_time!("One step");

        f(&mut system, &state_register);

        if state_register.0.borrow_mut().check_if_changed() {
            steps = 0;
        } else {
            steps += 1;
        }
    }

    system.set_system_state(state_register.minimal_state().unwrap().state);
    system.save_system(
        &format!("results/minimal_{}_{}.mfsys", lattice_name, system.system_size())
    );
}

pub fn runner_multi_thread<F: Fn(&mut System, &MutexStateRegisterer) + Sync + Send>(mut system: System, lattice_name: &str, thread_count: usize, mut f: F) {
    let mut state_register = MutexStateRegisterer(Mutex::new(StateRegistererInner::new()));
    let mut steps = 0;

    measure_time::print_time!("All");

    let mut systems = vec![system.clone(); thread_count];
    while !state_register.0.lock().unwrap().diff_between_mins(1e-8) || steps < 10 {
        measure_time::print_time!("One step");

        systems = systems
            .into_par_iter()
            .map(|mut system| {
                f(&mut system, &state_register);
                system
            })
            .collect();

        if state_register.0.lock().unwrap().check_if_changed() {
            steps = 0;
        } else {
            steps += 1;
        }
    }

    system.set_system_state(state_register.minimal_state().unwrap().state);
    system.save_system(
        &format!("results/minimal_{}_{}.mfsys", lattice_name, system.system_size())
    );
}

