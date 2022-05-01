use crate::generators::LatticeGenerator;
use bitvec::prelude::BitVec;
use element::Element;
use ordered_float::OrderedFloat;
use plotters::prelude::*;
use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::HashMap;
use num_traits::float::FloatCore;
use system::System;
use tap::Tap;

mod element;
mod generators;
mod system;



struct States {
    pair: (BitVec, f64, isize),
    actions: Vec<(BitVec, f64, StateActionKind)>,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum StateActionKind {
    Greedy,
    Step2,
    Step3,
    Minimize1,
    Minimize2,
}

impl States {
    pub fn new((cols, rows): (u64, u64)) -> Self {
        Self {
            pair: (Default::default(), f64::MAX, (cols * rows * 5) as isize),
            actions: vec![],
        }
    }

    pub fn save_state(&mut self, system: &System, action_type: StateActionKind) {
        self.actions
            .push((system.element_signs().clone(), system.energy(), action_type));
        if self.pair.1 > system.energy() {
            self.pair = (
                system.element_signs().clone(),
                system.energy(),
                system.magn(),
            );
        }
    }
}

// fn perebor(mut system: System, (cols, rows): (u64, u64), c: f64) {
//     let pair = (0..(1<<(system.elements.len())))
//         .into_iter()
//         .into_par_iter()
//         .fold(
//             || (vec![], f64::MAX),
//             |mut pair, i| {
//                 let mut system = system.clone();
//
//                 for b in 0..system.elements.len() {
//                     system.element_signs.set(b, i >> b & 1 == 1)
//                 }
//
//                 system.recalculate_energy();
//                 if pair.1 > system.energy {
//                     pair = (vec![system.element_signs.clone()], system.energy);
//                 }
//                 if pair.1 == system.energy {
//                     pair.0.push(system.element_signs.clone());
//                 }
//
//                 pair
//             }
//         )
//         .reduce(
//             || (vec![], f64::MAX),
//             |mut pair, pair2| {
//                 if pair.1 > pair2.1 {
//                     pair2
//                 } else if pair.1 == pair2.1 {
//                     pair.0.extend(pair2.0.into_iter());
//                     pair
//                 } else {
//                     pair
//                 }
//             }
//         );
//
//     for s in &pair.0 {
//         system.element_signs = s.clone();
//         system.recalculate_energy();
//         system.recalculate_m();
//
//         system.save_system(format!("{}_{}x{}_{}_par.mfsys", c, rows, cols, system.element_signs))
//     }
// }

fn gibrid(system: &mut System, states: &mut States) {
    let system_size = system.system_size();
    /*
       Реализация шага 1 из разработанного алгоритма
    */
    while !system.all_row_energies_negative() {
        system.greedy_step();
        states.save_state(system, StateActionKind::Greedy);
    }

    /*
       Реализация шага 2 из разработанного алгоритма
    */
    for i in 0..system_size {
        system.reverse_spin(i);
        states.save_state(system, StateActionKind::Step2);

        let sorted = system
            .row_sum_energies()
            .iter()
            .copied()
            .enumerate()
            .collect::<Vec<_>>()
            .tap_mut(|v| v.sort_by_key(|(_, x)| OrderedFloat(*x)));


        if sorted[0].0 == i {
            system.reverse_spin(i);
            let _ = states.actions.pop();
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
        //     let _ = states.actions.pop();
        //     continue;
        // }

        system.reverse_spin(sorted[0].0);
        states.save_state(system, StateActionKind::Step2);

        /*
            Реализация шага 1 внутри шага 2
        */
        if !system.all_row_energies_negative() {
            while !system.all_row_energies_negative() {
                system.greedy_step();
                states.save_state(system, StateActionKind::Step2);
            }
        }
    }

    // for count in 1..system_size {
    //     for i in 0..(system_size - count) {
    //         system.reverse_spins(
    //             (0..count).map(|j| if j % 11 == 0 { i + j } else { system_size - j })
    //         );
    //         while !system.all_row_energies_negative() {
    //             system.greedy_step();
    //             states.save_state(&system, StateActionKind::Greedy);
    //         }
    //         states.save_state(&system, StateActionKind::Step3)
    //     }
    // }
}

fn minimize_cells(system: &mut System, states: &mut States) {
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

            hash_map.entry(e.abs())
                .and_modify(|cells| cells.push((x, y)))
                .or_insert(Vec::new());
        }
    }

    let mut keys: Vec<_> = hash_map.keys().copied().collect();
    keys.sort_by_key(|x| Reverse(*x));

    for (key_index, key) in keys.iter().enumerate().take(20) {
        let greater_keys = &keys[0..key_index];

        for (x, y) in &hash_map[key] {
            let cant_change = rounded_matrix[*y].iter().any(|x| greater_keys.contains(x));
            if cant_change {
                let e1 = system.energy();
                system.reverse_spins([x, y].into_iter().copied());

                if e1 < system.energy() {
                    system.reverse_spins([x, y].into_iter().copied());
                } else {
                    states.save_state(system, StateActionKind::Minimize2);
                }
            }

            if system.energy_matrix()[*y][*x].is_sign_positive() {
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

                    states.save_state(system, StateActionKind::Minimize1);
                }
            } else {
                let e1 = system.energy();
                system.reverse_spins([x, y].into_iter().copied());

                if e1 < system.energy() {
                    system.reverse_spins([x, y].into_iter().copied());
                } else {
                    states.save_state(system, StateActionKind::Minimize2);
                }
            }
        }
    }
}

fn draw_state(states: &States, dir_name: &str) {
    let States { pair, actions, .. } = states;

    let image_path = format!("{dir_name}/chart.png");
    let root_drawing_area = BitMapBackend::new(&image_path, (1024, 768)).into_drawing_area();

    root_drawing_area.fill(&WHITE).unwrap();

    let mut ctx = ChartBuilder::on(&root_drawing_area)
        .caption("Ход работы алгоритма", ("Arial", 40))
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(0..actions.len(), (pair.1 * 1.1)..((pair.1 / 3.0).abs()))
        .unwrap();

    ctx.configure_mesh()
        .y_desc("Энергия")
        .x_desc("Номер переворота")
        .axis_desc_style(("sans-serif", 30))
        .draw()
        .unwrap();

    ctx.draw_series(LineSeries::new(
        actions.iter().enumerate().map(|(i, (_, e, _))| (i, *e)),
        &GREEN,
    ))
        .unwrap();

    ctx.draw_series(
        actions
            .iter()
            .enumerate()
            .filter(|(_, (_, _, a))| *a == StateActionKind::Greedy)
            .map(|(i, (_, e, _))| (i, *e))
            .map(|p| Circle::new(p, 2, &RED)),
    )
        .unwrap();

    ctx.draw_series(
        actions
            .iter()
            .enumerate()
            .filter(|(_, (_, _, a))| *a == StateActionKind::Step2)
            .map(|(i, (_, e, _))| (i, *e))
            .map(|p| Circle::new(p, 2, &BLUE)),
    )
        .unwrap();

    ctx.draw_series(
        actions
            .iter()
            .enumerate()
            .filter(|(_, (_, _, a))| *a == StateActionKind::Minimize1)
            .map(|(i, (_, e, _))| (i, *e))
            .map(|p| Circle::new(p, 2, &MAGENTA)),
    )
        .unwrap();

    ctx.draw_series(
        actions
            .iter()
            .enumerate()
            .filter(|(_, (_, _, a))| *a == StateActionKind::Minimize2)
            .map(|(i, (_, e, _))| (i, *e))
            .map(|p| Circle::new(p, 2, &CYAN)),
    )
        .unwrap();
}

fn main() {
    let cols = 2;
    let rows = 4;
    let c = 376.0;

    //Создание системы
    let mut system = LatticeGenerator::cairo(472.0, 344.0, c, 300.0, cols, rows);
    let mut states = States::new((cols, rows));

    // minimize_cells(&mut system, &mut states);
    gibrid(&mut system, &mut states);

    println!("{} {}", states.pair.0, states.pair.1);

    system.set_element_signs(states.pair.0.clone());

    let dir_name = format!(
        "results/cairo_{}_{}x{}_{}",
        system.system_size(),
        cols,
        rows,
        c
    );
    std::fs::create_dir_all(&dir_name).unwrap();

    system.save_system(format!("{dir_name}/algorithm.mfsys"));
    system.save_in_excel(format!("{dir_name}/algorithm.xlsx"));
    draw_state(&states, &dir_name);

    for action in states.actions {
        println!("{} {:?}", action.1, action.2);
    }
}
