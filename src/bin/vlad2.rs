use rand::{Rng, thread_rng};
use system_greedy::generators::LatticeGenerator;
use system_greedy::{gibrid, GibridState, prepare_state};
use system_greedy::runner::{Replicate, runner_multi_thread, StateRegisterer};

fn main() {
    let mut system = LatticeGenerator::trimer(225., 700., 20, 20);

    let GibridState { states_map, identity_map, radius } = prepare_state(&system);

    runner_multi_thread(system, "cairo", Replicate, 100, 1, |system, registerer, _| {
        let mut rng = thread_rng();
        for _ in 0..system.size() {
            let random = rng.gen_range(0..system.size());
            let cluster: Vec<_> = system.neighbors(random, radius).map(|(i, _)| i).collect();

            let identity = &identity_map[&random];
            let states = &states_map[identity];

            if states.is_empty() {
                continue;
            }

            system.set_spins(
                states[rng.gen_range(0..states.len())]
                    .state
                    .iter()
                    .enumerate()
                    .map(|(i, s)| (cluster[i], *s)),
            );
            registerer.register(system);

            {
                measure_time::print_time!("Gibrid");
                gibrid(system, registerer);
            }
        }
    });
}
