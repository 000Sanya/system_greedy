use crate::algorithn_state::{State, State2};
use crate::System;
use bitvec::prelude::{BitVec, Lsb0};
use bitvec::view::BitView;
use rayon::prelude::{ParallelBridge, ParallelIterator};

pub struct StateSaver {
    pub minimal_state: State,
    pub states: Vec<State2>,
}

impl StateSaver {
    pub fn new(count: usize) -> Self {
        Self {
            minimal_state: State::default(),
            states: Vec::with_capacity(count),
        }
    }

    pub fn save(&mut self, system: &System) {
        if system.energy() < 0.0
            && (self.minimal_state.energy - system.energy()).abs()
                <= (self.minimal_state.energy * 0.2).abs()
        {
            self.states.push(State2 {
                state: system.system_state().clone(),
                energy: system.energy(),
            });
        }
        if self.minimal_state.energy > system.energy() {
            self.minimal_state = State {
                state: system.system_state().clone(),
                energy: system.energy(),
                spin_excess: system.spin_excess(),
            };
        }
    }

    pub fn merge(&mut self, rhs: Self) {
        self.states.extend(rhs.states);
        if self.minimal_state.energy > rhs.minimal_state.energy {
            self.minimal_state.energy = rhs.minimal_state.energy;
        }
    }

    #[inline(always)]
    pub fn merged(mut self, rhs: Self) -> Self {
        self.merge(rhs);
        self
    }
}

fn grey_bitvec(g: BitVec) -> BitVec {
    let mut g1 = g.clone();
    g1.shift_right(1);
    g ^ g1
}

pub fn perebor_states(system: &System) -> Vec<(State, Vec<State2>)> {
    let system_size = system.system_size();
    let thread_count = rayon::current_num_threads();
    let state_count = 2usize.pow(system_size as u32);
    let block_size = state_count / thread_count;
    let remain = state_count % thread_count;

    let ranges = (0..thread_count).map(|i| {
        let start = i * block_size + i.min(remain);
        let count = block_size + if i < remain { 1 } else { 0 };
        start..start + count
    });

    ranges
        .into_iter()
        .par_bridge()
        .map(move |r| {
            let mut system = system.clone();
            let mut states = StateSaver::new(r.len());
            let start = r.start;
            let bit_view = start
                .view_bits::<Lsb0>()
                .into_iter()
                .take(system_size)
                .collect();
            let bit_view = grey_bitvec(bit_view);
            system.set_system_state(bit_view);
            states.save(&system);

            for i in r.skip(1) {
                let index = i.trailing_zeros();
                system.reverse_spin(index as usize);
                states.save(&system);
            }
            (states.minimal_state, states.states)
        })
        .collect()
}
