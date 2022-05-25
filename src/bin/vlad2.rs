use std::cmp::min;
use std::collections::HashMap;
use std::time::Instant;
use bitvec::bitvec;
use bitvec::prelude::BitVec;
use bitvec::prelude::Lsb0;
use bitvec::view::BitView;
use ordered_float::OrderedFloat;
use rand::{Rng, thread_rng};
use rayon::prelude::{IntoParallelIterator, ParallelBridge, ParallelIterator};
use tap::Tap;
use system_greedy::algorithn_state::{AlgorithmState, State, State2, StepKind};
use system_greedy::generators::LatticeGenerator;
use system_greedy::{gibrid_cluster, gibrid, export_csv};
use system_greedy::element::Element;
use system_greedy::perebor::perebor_states;
use system_greedy::system::System;
use system_greedy::system_part::get_part_from_system;

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

    let ranges = (0..thread_count)
        .map(|i| {
            let start = i * block_size + i.min(remain);
            let count = block_size + if i < remain { 1 } else { 0 };
            start..start + count
        });

    ranges.into_iter()
        .par_bridge()
        .map(move |r| {
            let mut system = system.clone();
            let mut states = AlgorithmState::new();
            let start = r.start;
            let bit_view = start.view_bits::<Lsb0>()
                .into_iter()
                .take(cluster.len())
                .collect();
            let bit_view = grey_bitvec(bit_view);
            let mut state = system.system_state().clone();
            bit_view.into_iter()
                .take(cluster.len())
                .enumerate()
                .for_each(|(i, s)| state.set(cluster[i], s));
            system.set_system_state(state);
            states.save_step_state2(&system, StepKind::Minimize1);

            for i in r.skip(1) {
                let index = i.trailing_zeros();
                system.reverse_spin(cluster[index as usize]);
                states.save_step_state2(&system, StepKind::Minimize1);
            }

            states.minimal_state
        })
        .min_by_key(|state| OrderedFloat(state.energy))
        .unwrap()
}

fn get_states(system: &System, radius: f64) -> (HashMap<Vec<Element>, Vec<State2>>, HashMap<usize, Vec<Element>>) {
    let mut states_map = HashMap::new();
    let mut identity_map = HashMap::new();

    for i in 0..system.system_size() {
        let neighbors: Vec<_> = system.neighbors2(i, radius).map(|(i, _)| i).collect();

        let part = get_part_from_system(system, &neighbors);
        if !states_map.contains_key(&part) {
            println!("Perebor system for {}, states: {}", i, states_map.len());
            let system = System::new(part.clone());
            let states = perebor_states(&system);

            let min = states.iter().min_by_key(|(s, _)| OrderedFloat(s.energy)).map(|(s, _)| s.energy).unwrap();
            let diff = min.abs() * 2.0 * 0.2;

            let states: Vec<_> = states
                .into_par_iter()
                .map(|states| {
                    let mut v = Vec::with_capacity(2usize.pow(20));
                    v.extend(
                        states.1
                            .into_iter()
                            .filter(|state| (state.energy - min).abs() <= diff)
                    );
                    v
                })
                .reduce(
                    || Vec::new(),
                    |mut gv, v| gv.tap_mut(|gv| gv.extend(v))
                );
            states_map.insert(part.clone(), states);
            println!("Stop perebor system for {}, states: {}", i, states_map.len());
        }
        identity_map.insert(i, part);
    }

    (states_map, identity_map)
}

fn main() {
    let cols = 4;
    let rows = 4;
    let c = 376.0;

    let steps = 3;

    let mut system = LatticeGenerator::cairo(472.0, 344.0, c, 300.0, cols, rows);
    let (mut system, gs) = export_csv("input/trim1200.csv");
    dbg!(system.system_size());

    let mut min = usize::MAX;
    let mut max = usize::MIN;

    let mut curr_e = f64::MAX;
    let mut prev_e;

    let mut curr_state = None;

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

    let (states_map, identity_map) = get_states(&system, raduis);

    {
        let mut algorithm_state = AlgorithmState::new();
        gibrid(&mut system, &mut algorithm_state);

        system.set_system_state(algorithm_state.minimal_state.state);
    }

    let mut rng = thread_rng();

    loop {
        prev_e = curr_e;
        for _ in 0..system.system_size() {
            let random = rng.gen_range(0..system.system_size());
            let cluster: Vec<_> = system.neighbors(random, raduis).map(|(i, _)| i).collect();

            let identity = &identity_map[&random];
            let states = &states_map[identity];

            if states.len() < 1 {
                continue
            }

            measure_time::print_time!("All time");

            let mut algorithm_state = AlgorithmState::new();

            system.set_spins(
                states[rng.gen_range(0..states.len())].state.iter().enumerate().map(|(i, s)| (cluster[i], *s))
            );
            algorithm_state.save_step_state2(&system, StepKind::Minimize1);

            if let Some(current_minimum) = algorithm_state.consume_minimal_state() {
                if current_minimum.energy <= curr_e {
                    curr_e = current_minimum.energy;
                    curr_state = Some(current_minimum.clone());
                    println!("{}", curr_e);
                    if curr_e < -1.55 {
                        let mut system = system.clone();
                        system.set_system_state(current_minimum.state.clone());
                        system.save_system(format!("results/min_{}.mfsys", current_minimum.energy));
                    }
                }
            }

            {
                measure_time::print_time!("Gibrid time");
                gibrid(&mut system, &mut algorithm_state);
            }

            if let Some(current_minimum) = algorithm_state.consume_minimal_state() {
                if current_minimum.energy <= curr_e {
                    curr_e = current_minimum.energy;
                    curr_state = Some(current_minimum.clone());
                    println!("gibrid {}", curr_e);
                    if curr_e < -1.55 {
                        let mut system = system.clone();
                        system.set_system_state(current_minimum.state.clone());
                        system.save_system(format!("results/min_{}.mfsys", system.energy()));
                    }
                }
            }

            if let Some(state) = curr_state.as_ref() {
                system.set_system_state(state.state.clone())
            }
        }

        let mut algorithm_state = AlgorithmState::new();
        gibrid(&mut system, &mut algorithm_state);

        if let Some(current_minimum) = algorithm_state.consume_minimal_state() {
            if current_minimum.energy <= curr_e {
                curr_e = current_minimum.energy;
                curr_state = Some(current_minimum.clone());
                println!("gibrid2 {}", curr_e);
                if curr_e < -1.55 {
                    let mut system = system.clone();
                    system.set_system_state(current_minimum.state.clone());
                    system.save_system(format!("results/min_{}.mfsys", system.energy()));
                }
            }
        }

        if let Some(state) = curr_state.as_ref() {
            system.set_system_state(state.state.clone())
        }

        println!("tick");

        if (curr_e - prev_e).abs() <= 1e-8 {
            break;
        }
    }

    println!("{}", curr_state.unwrap().state);
}