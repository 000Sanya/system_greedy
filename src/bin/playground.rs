use std::cell::RefCell;
use plotters::prelude::{BitMapBackend, EmptyElement, IntoDrawingArea, RED};
use system_greedy::generators::LatticeGenerator;
use system_greedy::greedy;
use system_greedy::runner::{RefCellStateRegisterer, StateRegistererInner};
use system_greedy::system::{Orientation, Vec2};
use system_greedy::utils::generate_arrow;

fn main() {
    let mut system = LatticeGenerator::trimer(450. / 2., 625., 32, 32);

    for y in 0..32 {
        for x in 0..32 {
            let index = (y * 32 + x) * 3;

            if y % 2 != 0 {
                system.set_spin_orientation(index + 0, Orientation::Down);
                system.set_spin_orientation(index + 1, Orientation::Down);
                system.set_spin_orientation(index + 2, Orientation::Down);
            } else {
                system.set_spin_orientation(index + 0, Orientation::Up);
                system.set_spin_orientation(index + 1, Orientation::Down);
                system.set_spin_orientation(index + 2, Orientation::Up);
            }
        }
    }


    system.save_mfsys("results/replicate_2_1.mfsys");
}
