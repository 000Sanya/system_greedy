use rand::{Rng, thread_rng};
use system_greedy::generators::LatticeGenerator;
use system_greedy::{gibrid, gibrid2, GibridState, greedy, prepare_state};
use system_greedy::perebor::perebor_one_thread;
use system_greedy::runner::{Replicate, runner_multi_thread};

fn main() {
    let mut system = LatticeGenerator::trimer(225., 625., 41, 41);

    println!("Preparing state");
    let gibrid_state = prepare_state(&system);

    println!("Start find");
    let state = runner_multi_thread(system.clone(), Replicate, /*MK-steps*/ 10, /*threds*/ 16, |system, registerer, _| {
        gibrid2(system, registerer, &gibrid_state);
        println!("Step finished");
    });
    dbg!(state.energy);

    system.set_system_state(state.state);
    system.save_mfsys(
        &format!("results/minimal_{}", state.energy)
    );
}
