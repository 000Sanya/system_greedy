use num_traits::Zero;
use plotters::prelude::{BitMapBackend, BLACK, Circle, DrawingBackend, EmptyElement, IntoDrawingArea, RGBColor, ShapeStyle, WHITE};
use system_greedy::generators::LatticeGenerator;
use system_greedy::system::System;
use system_greedy::utils::generate_arrow;

fn main() {
    // let system = System::load_mfsys("results/trim_1.mfsys");
    // let system = System::load_mfsys("results/minimal_trim_1200.mfsys");
    let system = System::load_mfsys("results/minimal_trim_75.mfsys");
    let system = System::load_mfsys("results/minimal_625_3072_-5.532045015143138.mfsys");
    // let system = System::load_mfsys("results/replicate_2.mfsys");
    // let system = System::load_mfsys("input/trimer_N1200_b700.mfsys");

    // let system = LatticeGenerator::cairo(472.0, 344.0, 376., 300.0, 2, 2);

    let max_x = system.elements().iter().max_by_key(|e| e.pos.x).map(|e| e.pos.x).unwrap().0 as u32 / 5;
    let max_y = system.elements().iter().max_by_key(|e| e.pos.y).map(|e| e.pos.y).unwrap().0 as u32 / 5;

    let mut plotter = BitMapBackend::new("results/minimal_625_3072_-5.532045015143138.png", (max_x + 200, max_y + 200)).into_drawing_area();
    plotter.fill(&RGBColor(127, 127, 127)).unwrap();
    // plotter.fill(&WHITE).unwrap();

    for (i, element) in system.elements().iter().enumerate() {
        let mut orient = 1;

        if element.magn.y.is_sign_negative() {
            orient *= -1;
        } else if element.magn.y.is_zero() && element.magn.x.is_sign_negative() {
            orient *= -1;
        }

        if system.system_state()[i] {
            orient *= -1;
        }

        let direction = if system.system_state()[i] {
            element.magn.map(|x| x.0) * -1.0
        } else {
            element.magn.map(|x| x.0)
        };

        let color = if orient > 0 {
            &BLACK
        } else {
            &WHITE
        };

        plotter.draw(
            &(
                EmptyElement::at(((element.pos.x.0) as i32 / 5 + 100, (element.pos.y.0 + 100.) as i32 / 5 + 100))
                    + Circle::new((0, 0), 10, ShapeStyle::from(color).filled())
                    // + generate_arrow(38., 7., 14., 20., direction, 0.8, color)
            )
        ).unwrap();
    }

    println!("TEST 3");

    plotter.present().unwrap();
}