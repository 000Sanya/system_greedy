use crate::generators::LatticeGenerator;
use bitvec::prelude::BitVec;
use element::Element;
use ordered_float::OrderedFloat;
use plotters::prelude::*;
use rayon::prelude::*;
use std::cmp::Reverse;
use std::collections::HashMap;
use system::System;
use tap::Tap;

mod element;
mod generators;
mod system;

struct States {
    states: HashMap<BitVec, f64>,
    pair: (BitVec, f64, isize),
    actions: Vec<(BitVec, f64, StateAction)>,
}

#[derive(Eq, PartialEq)]
enum StateAction {
    Greedy,
    Step2,
    Step3,
}

impl States {
    pub fn new((cols, rows): (u64, u64)) -> Self {
        Self {
            states: Default::default(),
            pair: (Default::default(), f64::MAX, (cols * rows * 5) as isize),
            actions: vec![],
        }
    }

    pub fn save_state(&mut self, system: &System, action_type: StateAction) {
        self.actions
            .push((system.element_signs().clone(), system.energy(), action_type));
        if system.magn() <= self.pair.2.abs() {
            self.states
                .insert(system.element_signs().clone(), system.energy());
            if self.pair.1 > system.energy() {
                self.pair = (
                    system.element_signs().clone(),
                    system.energy(),
                    system.magn(),
                );
            }
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
        states.save_state(system, StateAction::Greedy);
    }

    /*
       Реализация шага 2 из разработанного алгоритма
    */
    for i in 0..system_size {
        system.reverse_spin(i);
        states.save_state(system, StateAction::Step2);

        // let sorted = system
        //     .row_sum_energies
        //     .iter()
        //     .copied()
        //     .enumerate()
        //     .collect::<Vec<_>>()
        //     .tap_mut(|v| v.sort_by_key(|(_, x)| OrderedFloat(*x)));
        //
        //
        // if sorted[0].0 == i {
        //     system.reverse_spin(i);
        //     let _ = states.actions.pop();
        //     continue;
        // }

        let sorted = system
            .row_sum_energies()
            .iter()
            .copied()
            .enumerate()
            .collect::<Vec<_>>()
            .tap_mut(|v| v.sort_by_key(|(_, x)| Reverse(OrderedFloat(*x))));

        if sorted[0].0 == i || sorted[0].1.is_sign_negative() {
            system.reverse_spin(i);
            let _ = states.actions.pop();
            continue;
        }

        system.reverse_spin(sorted[0].0);
        states.save_state(system, StateAction::Step2);

        /*
            Реализация шага 1 внутри шага 2
        */
        if !system.all_row_energies_negative() {
            while !system.all_row_energies_negative() {
                system.greedy_step();
                states.save_state(system, StateAction::Step2);
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
    //             states.save_state(&system, StateAction::Greedy);
    //         }
    //         states.save_state(&system, StateAction::Step3)
    //     }
    // }
}

fn main() {
    let cols = 4;
    let rows = 4;
    let c = 376.0;

    //Создание системы
    let mut system = LatticeGenerator::cairo(472.0, 344.0, c, 300.0, cols, rows);
    let mut states = States::new((cols, rows));

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

    let image_path = format!("{dir_name}/chart.png");
    let root_drawing_area = BitMapBackend::new(&image_path, (1024, 768)).into_drawing_area();

    root_drawing_area.fill(&WHITE).unwrap();

    let States { pair, actions, .. } = states;

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
            .filter(|(_, (_, _, a))| *a == StateAction::Greedy)
            .map(|(i, (_, e, _))| (i, *e))
            .map(|p| Circle::new(p, 2, &RED)),
    )
    .unwrap();

    ctx.draw_series(
        actions
            .iter()
            .enumerate()
            .filter(|(_, (_, _, a))| *a == StateAction::Step2)
            .map(|(i, (_, e, _))| (i, *e))
            .map(|p| Circle::new(p, 2, &BLUE)),
    )
    .unwrap();

    ctx.draw_series(
        actions
            .iter()
            .enumerate()
            .filter(|(_, (_, _, a))| *a == StateAction::Step3)
            .map(|(i, (_, e, _))| (i, *e))
            .map(|p| Circle::new(p, 2, &MAGENTA)),
    )
    .unwrap();
}
