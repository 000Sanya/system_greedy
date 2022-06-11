pub mod element;
pub mod generators;
pub mod perebor;
pub mod system;
pub mod utils;
pub mod runner;
pub mod metropolis;
pub mod matrix;

use bitvec::prelude::BitVec;
use element::Element;
use ordered_float::OrderedFloat;
use plotters::prelude::*;

use std::cmp::Reverse;
use std::collections::{BTreeSet, HashMap};
use system::System;
use tap::Tap;

use crate::system::Vec2;
use rand::prelude::SliceRandom;
use rand::{Rng, thread_rng};
use rayon::prelude::{IntoParallelIterator, ParallelIterator};
use crate::perebor::perebor_states;
use crate::runner::{State, StateRegisterer};
use crate::utils::get_part_from_system;

pub fn greedy(system: &mut System, registerer: &impl StateRegisterer) {
    while let Some((index, _)) = system.row_energies().iter().copied()
        .enumerate()
        .filter(|x| !x.1.is_sign_negative())
        .max_by_key(|(_, x)| OrderedFloat(*x))
    {
        system.reverse_spin(index);
        registerer.register(system);
    }
}

pub fn gibrid(system: &mut System, registerer: &impl StateRegisterer) {
    let system_size = system.size();

    greedy(system, registerer);

    let mut indexes: Vec<_> = (0..system_size).collect();
    indexes.shuffle(&mut thread_rng());
    for i in &indexes {
        let i = *i;
        system.reverse_spin(i);

        let elem = system.row_energies().iter()
            .copied()
            .enumerate()
            .min_by_key(|(_, e)| OrderedFloat(*e))
            .unwrap();

        if elem.0 == i {
            system.reverse_spin(i);
            continue;
        }

        registerer.register(system);

        system.reverse_spin(elem.0);
        registerer.register(system);

        greedy(system, registerer);
    }
}

fn get_states(
    system: &System,
    radius: f64,
) -> (
    HashMap<Vec<Element>, Vec<State>>,
    HashMap<usize, Vec<Element>>,
) {
    let mut states_map = HashMap::new();
    let mut identity_map = HashMap::new();

    for i in 0..system.size() {
        let neighbors: Vec<_> = system.neighbors2(i, radius).map(|(i, _)| i).collect();


        let part = get_part_from_system(system, &neighbors);
        if !states_map.contains_key(&part) {
            let system = System::new(part.clone());
            let states = perebor_states(&system);

            let min = states
                .iter()
                .min_by_key(|(s, _)| OrderedFloat(s.energy))
                .map(|(s, _)| s.energy)
                .unwrap();
            let diff = min.abs() * 2.0 * 0.2;

            let states: Vec<_> = states
                .into_par_iter()
                .map(|states| {
                    let mut v = Vec::with_capacity(2usize.pow(20));
                    v.extend(
                        states
                            .1
                            .into_iter()
                            .filter(|state| (state.energy - min).abs() <= diff),
                    );
                    v
                })
                .reduce(Vec::new, |gv, v| gv.tap_mut(|gv| gv.extend(v)));
            states_map.insert(part.clone(), states);
        }
        identity_map.insert(i, part);
    }

    (states_map, identity_map)
}


pub struct GibridState {
    pub states_map: HashMap<Vec<Element>, Vec<State>>,
    pub identity_map: HashMap<usize, Vec<Element>>,
    pub radius: f64,
}

pub fn prepare_state(system: &System) -> GibridState {
    let mut min = usize::MAX;
    let mut max = usize::MIN;

    let mut radius = 0.0;
    let rnd_count = 20;

    let max_radius = system.max_radius();

    while max <= rnd_count && radius < max_radius {
        for i in 0..system.size() {
            let count = system.neighbors(i, radius).count();
            min = min.min(count);
            max = max.max(count)
        }
        radius += 10.0;
    }

    let (states_map, identity_map) = get_states(&system, radius);

    GibridState { states_map, identity_map, radius }
}

pub fn gibrid2(system: &mut System, registerer: &impl StateRegisterer, state: &GibridState) {
    let GibridState { states_map, identity_map, radius } = state;

    let size = system.size();
    let size10 = size / 10;

    let mut rng = thread_rng();
    for step in 0..system.size() {

        if (step + 1) % size10 == 0 {
            println!("Step {} / {}", step + 1, size);
        }

        let random = rng.gen_range(0..system.size());
        let cluster: Vec<_> = system.neighbors(random, *radius).map(|(i, _)| i).collect();

        let identity = &identity_map[&random];
        let states = &states_map[identity];

        if states.is_empty() {
            continue;
        }

        system.set_spins(
            states[rng.gen_range(0..states.len())]
                .state
                .iter()
                .enumerate()
                .map(|(i, s)| (cluster[i], *s)),
        );
        registerer.register(system);
        gibrid(system, registerer);
    }
}