use std::collections::{BTreeSet, HashMap};
use bitvec::prelude::BitVec;
use itertools::Itertools;
use ordered_float::OrderedFloat;
use rand::prelude::SliceRandom;
use rand::thread_rng;
use system_greedy::perebor_gem;
use system_greedy::system::System;

fn main() {
    let mut default_matrix = vec![
        vec![0.0, -4.0,  4.0, -4.0,  4.0],
        vec![0.0,  0.0,  1.0, -1.0,  2.0],
        vec![0.0,  0.0,  0.0,  2.0, -1.0],
        vec![0.0,  0.0,  0.0,  0.0, -3.0],
        vec![0.0,  0.0,  0.0,  0.0,  0.0],
    ];

    let mut default_matrix = vec![
        vec![ 0.0, -1.0, -1.0, -1.0, -1.0],
        vec![-1.0,  0.0, -1.0, -1.0, -1.0],
        vec![-1.0, -1.0,  0.0, -1.0, -1.0],
        vec![-1.0, -1.0, -1.0,  0.0, -1.0],
        vec![-1.0, -1.0, -1.0, -1.0,  0.0],
    ];

    let size = default_matrix.len();
    let indexes: Vec<_> = (0..size).flat_map(|y| ((y+1)..size).map(move |x| (y, x))).collect();
    let count = indexes.len();

    let mut gem_sizes = Vec::new();
    let mut min_es = Vec::new();
    let mut max_es = Vec::new();
    let mut min_max_es = Vec::new();
    for c in 1..=count {
        for comb in indexes.iter().combinations(c) {
            let mut default_matrix = default_matrix.clone();

            for (y, x) in comb {
                default_matrix[*y][*x] = 1.0;
            }

            let size = default_matrix.len();

            for y in 0..size {
                for x in (y+1)..size {
                    default_matrix[x][y] = default_matrix[y][x];
                }
            }

            let mut system = System::from_default_matrix(default_matrix.clone());
            let gem = perebor_gem(system.clone());
            save_in_csv(&default_matrix, &gem);

            gem_sizes.push((c, gem.len()));

            // {
            //     let mut gem: Vec<_> = gem.clone().into_iter().map(|((e, m), (g, states))| (e, m, g)).collect();
            //     gem.sort_by_key(|(e, m, _)| (*m, *e));
            //
            //     for (e, m, g) in &gem {
            //         println!("{}\t{}\t{}", m, e, g);
            //     }
            // }

            let (min_e, states) = gem.iter()
                .min_by_key(|((e, _), _)| *e)
                .map(|((e, _), (_, states))| (e, states))
                .unwrap();

            min_es.push((c, *min_e));

            // println!("min e: {:?}, {}", min_e, states[0]);

            let (max_e, states) = gem.iter()
                .max_by_key(|((e, _), _)| *e)
                .map(|((e, _), (_, states))| (e, states))
                .unwrap();

            max_es.push((c, *max_e));
            min_max_es.push((c, *min_e, *max_e));

            // println!("max e: {:?}, {}", max_e, states[0]);
            // println!("Sum: {:?}", min_e.abs() + max_e.abs());
        }
    }

    let gem_sizes: BTreeSet<_> = gem_sizes.into_iter().collect();
    let min_es: BTreeSet<_> = min_es.into_iter().collect();
    let max_es: BTreeSet<_> = max_es.into_iter().collect();
    let min_max_es: BTreeSet<_> = min_max_es.into_iter().collect();

    for (c, min, max) in min_max_es {
        println!("{} {} {}", c, min, max);
    }
}

fn save_in_csv(default_matrix: &Vec<Vec<f64>>, gem: &HashMap<(OrderedFloat<f64>, i32), (usize, Vec<BitVec>)>) {
    use std::fs::File;
    use std::io::Write;

    fn get_epoch_ms() -> u128 {
        use std::time::{SystemTime, UNIX_EPOCH};

        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis()
    }

    let dir = format!("results/exp/{}", get_epoch_ms());
    std::fs::create_dir_all(&dir).unwrap();

    let mut matrix_file = File::create(&format!("{}/matrix.csv", &dir)).unwrap();
    let size = default_matrix.len();
    for y in 0..size {
        for x in 0..size {
            write!(matrix_file, "{}", default_matrix[y][x]);
            if x < size - 1 {
                write!(matrix_file, ",");
            }
        }
        writeln!(matrix_file, "");
    }

    let mut gem_file  = File::create(&format!("{}/gem.csv", &dir)).unwrap();

    let mut gem: Vec<_> = gem.into_iter().map(|((e, m), (g, states))| (e, m, g)).collect();
    gem.sort_by_key(|(e, m, _)| (*m, *e));

    writeln!(gem_file, "M,E,G");
    for (e, m, g) in &gem {
        writeln!(gem_file, "{},{},{}", m, e, g);
    }
    writeln!(gem_file, ",,{}", gem.iter().map(|(_, _, g)| **g).sum::<usize>());

    let mut stats_file  = File::create(&format!("{}/stats.csv", &dir)).unwrap();
    let min_e = gem.iter().min_by_key(|(e, _, _)| *e).map(|(e, _, _)| e.0).unwrap();
    let max_e = gem.iter().max_by_key(|(e, _, _)| *e).map(|(e, _, _)| e.0).unwrap();

    writeln!(stats_file, "Min E,Max E,Sum");
    writeln!(stats_file, "{},{},{}", min_e, max_e, min_e.abs() + max_e.abs());
}