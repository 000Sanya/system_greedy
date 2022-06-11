use std::cell::RefCell;
use plotters::prelude::{BitMapBackend, EmptyElement, IntoDrawingArea, RED};
use system_greedy::generators::LatticeGenerator;
use system_greedy::greedy;
use system_greedy::runner::{RefCellStateRegisterer, StateRegistererInner};
use system_greedy::system::Vec2;
use system_greedy::utils::generate_arrow;

fn main() {
    let mut system = LatticeGenerator::trimer(450. / 2., 700., 20, 20);

    for i in (0..system.size()).filter(|x| x % 3 != 1) {
        system.reverse_spin(i);
    }

    let mut registerer = RefCellStateRegisterer(RefCell::new(StateRegistererInner::new()));
    greedy(&mut system, &mut registerer);

    system.save_mfsys("results/replicate_2.mfsys");
}
