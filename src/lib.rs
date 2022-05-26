pub mod element;
pub mod generators;
pub mod perebor;
pub mod system;
pub mod system_part;
pub mod utils;
pub mod runner;

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
use rand::thread_rng;
use crate::runner::StateRegisterer;

pub fn greedy(system: &mut System, registerer: &impl StateRegisterer) {
    while !system.row_energies().iter().all(|x| x.is_sign_negative()) {
        let index = system
            .row_energies()
            .iter()
            .enumerate()
            .max_by_key(|(_, x)| OrderedFloat(**x))
            .unwrap()
            .0;
        system.reverse_spin(index);
        registerer.register(system);
    }
}

pub fn gibrid(system: &mut System, registerer: &impl StateRegisterer) {
    let system_size = system.system_size();
    /*
       Реализация шага 1 из разработанного алгоритма
    */
    greedy(system, registerer);

    /*
       Реализация шага 2 из разработанного алгоритма
    */
    let mut indexes: Vec<_> = (0..system_size).collect();
    indexes.shuffle(&mut thread_rng());
    for i in &indexes {
        let i = *i;
        system.reverse_spin(i);

        let elem = system
            .row_energies()
            .iter()
            .copied()
            .enumerate()
            .min_by_key(|(_, e)| OrderedFloat(*e))
            .unwrap();

        if elem.0 == i {
            system.reverse_spin(i);
            continue;
        }

        // let elem = system
        //     .row_energies()
        //     .iter()
        //     .copied()
        //     .enumerate()
        //     .max_by_key(|(_, e)| OrderedFloat(*e))
        //     .unwrap();
        //
        //
        // if elem.0 == i || elem.1.is_sign_negative() {
        //     system.reverse_spin(i);
        //     continue;
        // }

        registerer.register(system);

        system.reverse_spin(elem.0);
        registerer.register(system);

        /*
            Реализация шага 1 внутри шага 2
        */
        greedy(system, registerer);
    }
}

pub fn minimize_cells(system: &mut System, registerer: &impl StateRegisterer) {
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

            hash_map
                .entry(OrderedFloat(e.abs()))
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

                    registerer.register(system);
                }
            } else {
                let e1 = system.energy();
                system.reverse_spins([x, y].into_iter().copied());

                if e1 < system.energy() {
                    system.reverse_spins([x, y].into_iter().copied());
                } else {
                    registerer.register(system);
                }
            }
        }
    }
}

pub fn import_csv(filepath: &str) -> (System, BitVec) {
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
