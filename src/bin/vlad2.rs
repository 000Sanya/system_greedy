use std::collections::HashMap;

use bitvec::prelude::BitVec;
use bitvec::prelude::Lsb0;
use bitvec::view::BitView;
use ordered_float::OrderedFloat;
use rand::{thread_rng, Rng};
use rayon::prelude::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use system_greedy::element::Element;
use system_greedy::generators::LatticeGenerator;
use system_greedy::perebor::perebor_states;
use system_greedy::system::System;
use system_greedy::system_part::get_part_from_system;
use system_greedy::{import_csv, gibrid};
use tap::Tap;
use system_greedy::runner::{runner_multi_thread, runner_one_thread, State, StateRegisterer, StateRegistererInner};

fn grey_bitvec(g: BitVec) -> BitVec {
    let mut g1 = g.clone();
    g1.shift_right(1);
    g ^ g1
}

fn perebor(system: &System, cluster: &[usize]) -> State {
    let thread_count = rayon::current_num_threads();
    let state_count = 2usize.pow(cluster.len() as u32);
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
            let mut registerer = StateRegistererInner::new();
            let start = r.start;
            let bit_view = start
                .view_bits::<Lsb0>()
                .into_iter()
                .take(cluster.len())
                .collect();
            let bit_view = grey_bitvec(bit_view);
            let mut state = system.system_state().clone();
            bit_view
                .into_iter()
                .take(cluster.len())
                .enumerate()
                .for_each(|(i, s)| state.set(cluster[i], s));
            system.set_system_state(state);
            registerer.register(&system);

            for i in r.skip(1) {
                let index = i.trailing_zeros();
                system.reverse_spin(cluster[index as usize]);
                registerer.register(&system);
            }

            registerer.minimal_state().unwrap()
        })
        .min_by_key(|state| OrderedFloat(state.energy))
        .unwrap()
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

    for i in 0..system.system_size() {
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

fn main() {
    let cols = 4;
    let rows = 4;
    let c = 376.0;

    let system = LatticeGenerator::cairo(472.0, 344.0, c, 300.0, cols, rows);
    let (mut system, _gs) = import_csv("input/trim1200.csv");
    dbg!(system.system_size());

    let mut min = usize::MAX;
    let mut max = usize::MIN;

    let mut raduis = 0.0;
    let rnd_count = 20;

    while max <= rnd_count {
        for i in 0..system.system_size() {
            let count = system.neighbors(i, raduis).count();
            min = min.min(count);
            max = max.max(count)
        }
        raduis += 10.0;
    }

    dbg!(min);
    dbg!(max);

    let (states_map, identity_map) = {
        measure_time::print_time!("Prepare state");
        get_states(&system, raduis)
    };

    runner_multi_thread(system, "trim", 6, |system, registerer| {
        let mut rng = thread_rng();
        for _ in 0..system.system_size() {
            let random = rng.gen_range(0..system.system_size());
            let cluster: Vec<_> = system.neighbors(random, raduis).map(|(i, _)| i).collect();

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
            system.set_system_state(registerer.minimal_state().unwrap().state)
        }
    });
}
