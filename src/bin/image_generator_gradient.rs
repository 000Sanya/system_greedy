use ordered_float::OrderedFloat;
use plotters::prelude::{BitMapBackend, BLACK, Circle, DrawingBackend, EmptyElement, IntoDrawingArea, RGBColor, WHITE};
use system_greedy::generators::LatticeGenerator;
use system_greedy::system::System;
use system_greedy::utils::generate_arrow;

fn main() {
    // let system = System::load_mfsys("results/trim_1.mfsys");
    let system = System::load_mfsys("results/minimal_trim_1200.mfsys");
    // let system = System::load_mfsys("results/minimal_trim_36_2.mfsys");
    // let system = System::load_mfsys("results/minimal_trim_75.mfsys");
    // let system = System::load_mfsys("results/minimal_cairo_80.mfsys");
    // let system = System::load_mfsys("input/trimer_N1200_b700.mfsys");
    // let system = System::load_mfsys("results/replicate.mfsys");
    // let system = System::load_mfsys("results/replicate_2.mfsys");

    let max_x = system.elements().iter().max_by_key(|e| e.pos.x).map(|e| e.pos.x).unwrap().0 as u32 / 5;
    let max_y = system.elements().iter().max_by_key(|e| e.pos.y).map(|e| e.pos.y).unwrap().0 as u32 / 5;

    let max_energy = system.row_energies().iter().copied().max_by_key(|x| OrderedFloat(*x)).unwrap();
    let min_energy = system.row_energies().iter().copied().min_by_key(|x| OrderedFloat(*x)).unwrap();

    let gradient = move |energy: f64| {
        if energy.is_sign_positive() {
            let percent = energy / max_energy;
            let color = (230.0 * (1.0 - percent)) as u8;
            RGBColor(255, color, color)
        } else {
            let percent = (energy / min_energy).abs();
            let color = (230.0 * (1.0 - percent)) as u8;
            RGBColor(color, color, 255)
        }
    };

    let mut plotter = BitMapBackend::new("results/minimal_trim_1200_gradient.png", (max_x + 200, max_y + 200)).into_drawing_area();
    plotter.fill(&RGBColor(127, 127, 127)).unwrap();

    for (i, element) in system.elements().iter().enumerate() {
        let energy = system.row_energies()[i];
        let gradient = gradient(energy);

        let direction = if system.system_state()[i] {
            element.magn.map(|x| x.0) * -1.0
        } else {
            element.magn.map(|x| x.0)
        };

        plotter.draw(
            &( EmptyElement::at(((element.pos.x.0) as i32 / 5 + 100, (element.pos.y.0 + 100.) as i32 / 5 + 100))
                + generate_arrow(40., 7., 14., 20., direction, 0.8, &gradient)

            )
        ).unwrap();
    }

    println!("TEST 3");

    plotter.present().unwrap();
}