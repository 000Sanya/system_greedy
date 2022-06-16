use rand::{Rng, thread_rng};
use system_greedy::generators::LatticeGenerator;
use system_greedy::{gibrid, gibrid2, GibridState, greedy, prepare_state};
use system_greedy::perebor::perebor_one_thread;
use system_greedy::runner::{Replicate, runner_multi_thread, StateRegisterer};

#[derive(Debug, Clone, Copy)]
pub enum Lattice {
    Trim { size: usize, b: f64 },
    Cairo { size: usize, c: f64 },
}

fn cairo(size: usize, c: f64) -> Lattice {
    Lattice::Cairo { size, c }
}

fn trim(size: usize, b: f64) -> Lattice {
    Lattice::Trim { size, b }
}

fn main() {
    let lattices = [
        trim(20, 680.),
    ];

    for lattice in lattices {
        let mut system = match lattice {
            Lattice::Trim { size, b } => LatticeGenerator::trimer(225., b, size, size),
            Lattice::Cairo { size, c } => LatticeGenerator::cairo(472.0, 344.0, c, 300.0, size as u64, size as u64)
        };

        println!("Preparing state");
        let gibrid_state = prepare_state(&system);

        println!("Start find");
        let state = runner_multi_thread(system.clone(), Replicate, /*MK-steps*/ 10, /*threds*/ 16, |system, registerer, _| {
            gibrid2(system, registerer, &gibrid_state);
            dbg!(registerer.minimal_state().map(|x| x.energy));
            println!("Step finished");
        });
        dbg!(state.energy);

        system.set_system_state(state.state);

        match lattice {
            Lattice::Trim { size, b } => system.save_mfsys(
                &format!("results/minimal_trim_{}x{}_{}.mfsys", size, size, b)
            ),
            Lattice::Cairo { size, c } => system.save_mfsys(
                &format!("results/minimal_cairo_{}x{}_{}.mfsys", size, size, c)
            ),
        }
    }
}
