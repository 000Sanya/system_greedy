pub mod element;
pub mod generators;
pub mod system;
pub mod algorithn_state;
pub mod perebor;
pub mod system_part;

use crate::generators::LatticeGenerator;
use bitvec::prelude::BitVec;
use element::Element;
use ordered_float::OrderedFloat;
use plotters::prelude::*;
use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::{BTreeSet, HashMap, HashSet};
use std::time::Instant;
use system::System;
use tap::Tap;
use algorithn_state::{AlgorithmState, StepKind};
use crate::algorithn_state::Step;
use itertools::Itertools;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use crate::system::Vec2;

pub fn perebor_gem(mut system: System) -> HashMap<(OrderedFloat<f64>, i32), (usize, Vec<BitVec>)> {
    let mut gem: HashMap<(OrderedFloat<f64>, i32), (usize, Vec<BitVec>)> = HashMap::new();

    for state in 0..( 1 << system.system_size()) {
        let mut system = system.clone();

        let mut system_state = system.system_state().clone();
        for b in 0..system.system_size() {
            system_state.set(b, state >> b & 1 == 1);
        }
        system.set_system_state(system_state);

        let e = system.energy();
        let m = system.spin_excess();

        gem.entry((OrderedFloat(e), m))
            .and_modify(|(g, states)| {
                *g += 1;
                states.push(system.system_state().clone())
            })
            .or_insert((1usize, vec![system.system_state().clone()]));
    }

    gem
}

pub fn greedy(system: &mut System, states: &mut AlgorithmState) {
    while !system.row_energies().iter().all(|x| x.is_sign_negative()) {
        let index = system.row_energies()
            .iter()
            .enumerate()
            .max_by_key(|(_, x)| OrderedFloat(**x))
            .unwrap().0;
        system.reverse_spin(index);
        states.save_step_state2(system, StepKind::Greedy);
    }
}

pub fn greedy_cluster(system: &mut System, states: &mut AlgorithmState, cluster: &HashMap<usize, usize>) {
    while !system.row_energies()
        .iter()
        .enumerate()
        .filter(|(i, _)| cluster.contains_key(i))
        .all(|(_, x)| x.is_sign_negative())
    {
        let index = system.row_energies()
            .iter()
            .enumerate()
            .filter(|(i, _)| cluster.contains_key(i))
            .max_by_key(|(_, x)| OrderedFloat(**x))
            .unwrap().0;
        system.reverse_spin(index);
        states.save_step_state(system, StepKind::Greedy);
    }
}

pub fn gibrid_cluster(system: &mut System, states: &mut AlgorithmState, cluster: &[usize]) {
    let cluster_map: HashMap<_, _> = cluster.iter().enumerate().map(|(i, i2)| (*i2, i)).collect();

    let system_size = cluster.len();
    greedy_cluster(system, states, &cluster_map);


    let mut indexes: Vec<_> = cluster_map.keys().copied().collect();
    indexes.shuffle(&mut thread_rng());
    for i in &indexes {
        let i = *i;
        system.reverse_spin(i);

        let sorted = system
            .row_energies()
            .iter()
            .copied()
            .enumerate()
            .filter(|(i, _)| cluster_map.contains_key(i))
            .collect::<Vec<_>>()
            .tap_mut(|v| v.sort_by_key(|(_, x)| OrderedFloat(*x)));


        if sorted[0].0 == i {
            system.reverse_spin(i);
            continue;
        }

        // let sorted = system
        //     .row_sum_energies()
        //     .iter()
        //     .copied()
        //     .enumerate()
        //     .collect::<Vec<_>>()
        //     .tap_mut(|v| v.sort_by_key(|(_, x)| Reverse(OrderedFloat(*x))));
        //
        // if sorted[0].0 == i || sorted[0].1.is_sign_negative() {
        //     system.reverse_spin(i);
        //     continue;
        // }

        states.save_step_state(system, StepKind::Step2);
        system.reverse_spin(sorted[0].0);
        states.save_step_state(system, StepKind::Step2);

        /*
            Реализация шага 1 внутри шага 2
        */
        greedy_cluster(system, states, &cluster_map);
    }
}

pub fn gibrid(system: &mut System, states: &mut AlgorithmState) {
    let system_size = system.system_size();
    /*
       Реализация шага 1 из разработанного алгоритма
    */
    greedy(system, states);

    /*
       Реализация шага 2 из разработанного алгоритма
    */
    let mut indexes: Vec<_> = (0..system_size).collect();
    indexes.shuffle(&mut thread_rng());
    for i in &indexes {
        let i = *i;
        system.reverse_spin(i);

        let sorted = system
            .row_energies()
            .iter()
            .copied()
            .enumerate()
            .collect::<Vec<_>>()
            .tap_mut(|v| v.sort_by_key(|(_, x)| OrderedFloat(*x)));


        if sorted[0].0 == i {
            system.reverse_spin(i);
            continue;
        }

        // let sorted = system
        //     .row_sum_energies()
        //     .iter()
        //     .copied()
        //     .enumerate()
        //     .collect::<Vec<_>>()
        //     .tap_mut(|v| v.sort_by_key(|(_, x)| Reverse(OrderedFloat(*x))));
        //
        // if sorted[0].0 == i || sorted[0].1.is_sign_negative() {
        //     system.reverse_spin(i);
        //     continue;
        // }

        states.save_step_state2(system, StepKind::Step2);
        system.reverse_spin(sorted[0].0);
        states.save_step_state2(system, StepKind::Step2);

        /*
            Реализация шага 1 внутри шага 2
        */
        greedy(system, states);
    }
}

pub fn minimize_cells(system: &mut System, states: &mut AlgorithmState) {
    let mut hash_map: HashMap<_, Vec<(usize, usize)>> = HashMap::new();

    let round = |x: f64| (x * 1_0000000000.0).round() / 1_0000000000.0;

    let mut rounded_matrix = Vec::with_capacity(system.system_size());

    for y in 0..system.system_size() {
        rounded_matrix.push(Vec::new());
        for x in 0..system.system_size() {
            let e = OrderedFloat(round(system.energy_matrix()[y][x]));
            rounded_matrix[y].push(e);

            if e.abs() == 0.0 {
                continue;
            }

            hash_map.entry(OrderedFloat(e.abs()))
                .and_modify(|cells| cells.push((x, y)))
                .or_insert(Vec::new());
        }
    }

    let mut keys: Vec<_> = hash_map.keys().copied().collect();
    keys.sort_by_key(|x| Reverse(*x));

    for (key_index, key) in keys.iter().enumerate().take(5) {
        let greater_keys = &keys[0..key_index];

        let indexes: BTreeSet<_> = hash_map[key].iter().collect();
        let mut indexes: Vec<_> = indexes.into_iter().collect();
        indexes.shuffle(&mut thread_rng());
        for (x, y) in indexes.iter() {
            let cant_change = rounded_matrix[*y].iter().any(|x| greater_keys.contains(x));

            if system.energy_matrix()[*y][*x].is_sign_positive() && !cant_change {
                let old_e = system.energy();

                system.reverse_spin(*x);
                let e1 = system.energy();
                system.reverse_spin(*x);

                system.reverse_spin(*y);
                let e2 = system.energy();
                system.reverse_spin(*y);

                if old_e > e1 || old_e > e2 {
                    if e1 < e2 {
                        system.reverse_spin(*x)
                    } else {
                        system.reverse_spin(*y)
                    }

                    states.save_step_state(system, StepKind::Minimize1);
                }
            } else {
                let e1 = system.energy();
                system.reverse_spins([x, y].into_iter().copied());

                if e1 < system.energy() {
                    system.reverse_spins([x, y].into_iter().copied());
                } else {
                    states.save_step_state(system, StepKind::Minimize2);
                }
            }
        }
    }
}

pub fn draw_state(states: &AlgorithmState, dir_name: &str) {
    let AlgorithmState { minimal_state, steps, .. } = states;

    let image_path = format!("{dir_name}/chart.png");
    let root_drawing_area = BitMapBackend::new(&image_path, (1024, 768)).into_drawing_area();

    root_drawing_area.fill(&WHITE).unwrap();

    let mut ctx = ChartBuilder::on(&root_drawing_area)
        .caption("Ход работы алгоритма", ("Arial", 40u32))
        .set_label_area_size(LabelAreaPosition::Left, 40u32)
        .set_label_area_size(LabelAreaPosition::Bottom, 40u32)
        .build_cartesian_2d(0..steps.len(), (minimal_state.energy * 1.1)..((minimal_state.energy / 3.0).abs()))
        .unwrap();

    ctx.configure_mesh()
        .y_desc("Энергия")
        .x_desc("Номер переворота")
        .axis_desc_style(("sans-serif", 30))
        .draw()
        .unwrap();

    ctx.draw_series(
        LineSeries::new(
            steps.iter().enumerate().map(|(i, step)| (i, step.state.energy)),
            &GREEN,
        )
    )
        .unwrap();

    let mut draw_steps = |steps: &[Step], step_kind: StepKind, color: &RGBColor| {
        ctx.draw_series(
            steps
                .iter()
                .enumerate()
                .filter(|(_, step)| step.step_kind == step_kind)
                .map(|(i, step)| (i, step.state.energy))
                .map(|p| Circle::new(p, 2, color)),
        )
            .unwrap();
    };

    draw_steps(steps, StepKind::Greedy, &RED);
    draw_steps(steps, StepKind::Step2, &BLUE);
    draw_steps(steps, StepKind::Minimize1, &MAGENTA);
    draw_steps(steps, StepKind::Minimize2, &CYAN);
}

pub fn export_csv(filepath: &str) -> (System, BitVec) {
    let mut reader = csv::Reader::from_path(filepath).unwrap();
    let mut states = BitVec::new();

    let mut elements = Vec::new();
    for result in reader.records() {
        let record = result.unwrap();

        let _id: u64 = record[0].parse().unwrap();
        let x: f64 = record[1].parse().unwrap();
        let y: f64 = record[2].parse().unwrap();
        let _: f64 = record[3].parse().unwrap();
        let mx: f64 = record[4].parse().unwrap();
        let my: f64 = record[5].parse().unwrap();
        let _: f64 = record[6].parse().unwrap();
        let state: u8 = record[7].parse().unwrap();

        let pos = Vec2::new(x, y);
        let mut magn = Vec2::new(mx, my);
        if state == 1 {
            magn *= -1.0;
            states.push(true)
        } else {
            states.push(false)
        }

        elements.push(Element::new(pos, magn));
    }

    (System::new(elements), states)
}