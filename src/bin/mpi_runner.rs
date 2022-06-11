use bitvec::prelude::BitVec;
use mpi::traits::{Communicator, Root};
use rand::{Rng, thread_rng};
use vek::Vec2;
use system_greedy::element::Element;
use system_greedy::gibrid;
use system_greedy::runner::{runner_one_thread, StateRegisterer};
use system_greedy::system::System;

fn main() {
    let filename = "input/trimer_N1200_b700.mfsys";

    let universe = mpi::initialize().unwrap();
    let world = universe.world();
    let size = world.size();
    let rank = world.rank();

    let root_rank = 0;
    let root_process = world.process_at_rank(root_rank);

    let mut system: System;
    let mut elements_as_linear_array: Vec<f64>;
    let mut element_count: usize = 0;

    let mut state_count: usize = 0;
    let mut bits: Vec<usize>;

    if rank == root_rank {
        system = System::load_mfsys(filename);
        elements_as_linear_array = system.elements()
            .iter()
            .copied()
            .flat_map(|element| [element.pos.x.0, element.pos.y.0, element.magn.x.0, element.magn.y.0])
            .collect();
        element_count = elements_as_linear_array.len();

        root_process.broadcast_into(&mut element_count);
        root_process.broadcast_into(&mut elements_as_linear_array[..]);

        let mut state = system.system_state().clone();
        state.set_uninitialized(false);
        bits = state.into_vec();

        state_count = bits.len();
        root_process.broadcast_into(&mut state_count);
        root_process.broadcast_into(&mut bits[..]);
    } else {
        root_process.broadcast_into(&mut element_count);

        elements_as_linear_array = std::iter::repeat(0.0).take(element_count).collect();
        root_process.broadcast_into(&mut elements_as_linear_array[..]);

        root_process.broadcast_into(&mut state_count);

        bits = std::iter::repeat(0).take(state_count).collect();
        root_process.broadcast_into(&mut bits[..]);

        let elements: Vec<_> = elements_as_linear_array
            .chunks(4)
            .map(|b| Element::new(
                Vec2::new(b[0], b[1]),
                Vec2::new(b[2], b[3]),
            ))
            .collect();

        let mut state = BitVec::from_vec(bits);
        state.resize(elements.len(), false);

        system = System::new(elements);
        system.set_system_state(state);
    }

    runner_one_thread(system, "cairo", |system, registerer| {
        let mut rng = thread_rng();
        for _ in 0..system.size() {
            let random = rng.gen_range(0..system.size());
            let cluster: Vec<_> = system.neighbors(random, raduis).map(|(i, _)| i).collect();

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

            gibrid(system, registerer);
            system.set_system_state(registerer.minimal_state().unwrap().state)
        }
    });
}