use std::cmp::min;
use std::time::Instant;
use bitvec::bitvec;
use bitvec::prelude::BitVec;
use bitvec::prelude::Lsb0;
use bitvec::view::BitView;
use ordered_float::OrderedFloat;
use rand::{Rng, thread_rng};
use rayon::prelude::{ParallelBridge, ParallelIterator};
use system_greedy::algorithn_state::{AlgorithmState, State, StepKind};
use system_greedy::generators::LatticeGenerator;
use system_greedy::{gibrid_cluster, gibrid, export_csv};
use system_greedy::system::System;

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

fn main() {
    let cols = 2;
    let rows = 3;
    let c = 376.0;

    let steps = 3;

    // let mut system = LatticeGenerator::cairo(472.0, 344.0, c, 300.0, cols, rows);
    let (mut system, gs) = export_csv("results/trim1200.csv");

    let mut min = usize::MAX;
    let mut max = usize::MIN;

    let mut curr_e = f64::MAX;
    let mut prev_e;

    let mut curr_state = None;
    //
    let mut states = AlgorithmState::new();
    gibrid(&mut system, &mut states);

    loop {
        let mut raduis = 0.0;
        let rnd_count = thread_rng().gen_range(10..=40);

        loop {
            for i in 0..system.system_size() {
                let count = system.neighbors(i, raduis).count();
                min = min.min(count);
                max = max.max(count)
            }

            if max > rnd_count {
                break
            }

            raduis += 10.0;
        }

        dbg!(min);
        dbg!(max);

        prev_e = curr_e;
        for _ in 0..system.system_size() {
            let random = thread_rng().gen_range(0..system.system_size());
            let cluster: Vec<_> = system.neighbors(random, raduis).map(|(i, _)| i).collect();

            let current_minimum = perebor(&system, &cluster);
            system.set_system_state(current_minimum.state.clone());

            if current_minimum.energy <= curr_e {
                curr_e = current_minimum.energy;
                curr_state = Some(current_minimum);
                println!("{}", curr_e);
            }
        }

        if (curr_e - prev_e).abs() <= 1e-8 {
            break
        }
    }

    println!("{}", curr_state.unwrap().state);
}