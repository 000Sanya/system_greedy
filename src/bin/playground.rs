use std::iter::repeat;
use bitvec::access::BitAccess;
use bitvec::prelude::BitVec;
use system_greedy::generators::LatticeGenerator;
use system_greedy::system::System;

fn main() {
    let cols = 2;
    let rows = 2;
    let c = 376.0;

    let mut system = LatticeGenerator::cairo(472.0, 344.0, c, 300.0, cols, rows);
    minimize_by_best_state(&mut system);
}

fn minimize_by_best_state(system: &mut System) {
    let mut states: Vec<Vec<(bool, f64)>> = Vec::with_capacity(system.system_size() * 2);

    let abs_sum: f64 = system.energy_matrix()
        .into_iter()
        .flat_map(|r| r.into_iter().map(|e| e.abs()))
        .sum();

    let rows_percents: Vec<_> = system.energy_matrix()
        .into_iter()
        .map(|row| {
            (row.into_iter().map(|e| e.abs()).sum::<f64>() / abs_sum) * 100.0
        })
        .collect();

    dbg!(&rows_percents);

    for (r, row) in system.energy_matrix().into_iter().enumerate() {
        let mut best_state = Vec::with_capacity(system.system_size());

        for (i, cell) in row.into_iter().enumerate() {
            if r == i {
                best_state.push((system.system_state()[i], 0.0));
                continue
            }

            let percente = (cell.abs() / abs_sum) * 100.0;

            match cell {
                x if x.is_sign_positive() => best_state.push((system.system_state()[i], percente)),
                x if x.is_sign_negative() => best_state.push((!system.system_state()[i], percente)),
                _ => unreachable!()
            }
        }

        states.push(best_state);
    }
}