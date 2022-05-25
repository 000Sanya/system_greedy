use system_greedy::generators::LatticeGenerator;

fn main() {
    let cols = 2;
    let rows = 2;
    // let c = 376.0;
    //
    // let mut system = LatticeGenerator::cairo(472.0, 344.0, c, 300.0, cols, rows);
    // minimize_by_best_state(&mut system);

    // let b = 600.0;
    // let system = LatticeGenerator::trimer(450.0 / 2.0, b, rows, cols);
    // system.save_system("results/trimer.mfsys");

    let system = LatticeGenerator::wtf(700., rows, cols);
    system.save_system("results/wtf.mfsys");
}
