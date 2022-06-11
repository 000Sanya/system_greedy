use rand::{Rng, thread_rng};
use system_greedy::generators::LatticeGenerator;
use system_greedy::{gibrid, gibrid2, GibridState, greedy, prepare_state};
use system_greedy::perebor::perebor_one_thread;
use system_greedy::runner::{Replicate, runner_multi_thread, runner_one_thread, StateRegisterer};

fn main() {
    let mut system = LatticeGenerator::trimer(225., 700., 5, 5);

    let gibrid_state = prepare_state(&system);

    let state = runner_multi_thread(system.clone(), Replicate, /*MK-steps*/ 10, /*threds*/ 12, |system, registerer, _| {
        gibrid2(system, registerer, &gibrid_state);
    });
    dbg!(state.energy);
}
