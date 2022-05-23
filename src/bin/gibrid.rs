use bitvec::vec::BitVec;
use system_greedy::algorithn_state::AlgorithmState;
use system_greedy::generators::LatticeGenerator;
use system_greedy::{draw_state, gibrid, minimize_cells};
use system_greedy::element::Element;
use system_greedy::system::{System, Vec2};

fn main() {
    let cols = 4;
    let rows = 4;
    let c = 376.0;

    let mut system = LatticeGenerator::cairo(472.0, 344.0, c, 300.0, cols, rows);
    let mut states = AlgorithmState::new();
    // let (mut system, gs) = export_csv("results/trim1200.csv");
    // system.set_system_state(gs);

    let mut system = system.clone();

    loop {
        for _ in 0..5 {
            gibrid(&mut system, &mut states);
            minimize_cells(&mut system, &mut states);
        }

        if let Some(minimal_state) = states.consume_minimal_state() {
            println!("{}", minimal_state.energy);
            // control_system.set_system_state(states.minimal_state.state.clone());
            //
            // let dir_name = format!(
            //     "results/trim_{}",
            //     control_system.system_size(),
            // );
            // std::fs::create_dir_all(&dir_name).unwrap();
            //
            // control_system.save_system(format!("{dir_name}/algorithm.mfsys"));
            // control_system.save_in_excel(format!("{dir_name}/algorithm.xlsx"), 0.0);
        }
    }
    // draw_state(&states, &dir_name);
}


