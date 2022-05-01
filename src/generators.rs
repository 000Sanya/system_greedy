use crate::system::Vec2;
use crate::{Element, System};
use num_traits::Zero;

pub struct LatticeGenerator;

impl LatticeGenerator {
    pub fn cairo(a: f64, b: f64, c: f64, l: f64, cols: u64, rows: u64) -> System {
        let mut elements = Vec::with_capacity((cols * rows * 5) as usize);

        let sin60 = std::f64::consts::FRAC_PI_3.sin();
        let yd = (2.0 * a - c) / 2.0;
        let xd = yd / 2.0;

        let points = [
            Vec2::new(0.0, 0.0),
            Vec2::new(xd + b / 2.0, yd * sin60),
            Vec2::new(xd + b / 2.0, -yd * sin60),
            Vec2::new(-xd - b / 2.0, -yd * sin60),
            Vec2::new(-xd - b / 2.0, yd * sin60),
        ];

        let magn = [
            Vec2::new(l * 1.0, 0.0),
            Vec2::new(l * 0.5, l * sin60),
            Vec2::new(l * 0.5, l * -sin60),
            Vec2::new(l * -0.5, l * -sin60),
            Vec2::new(l * -0.5, l * sin60),
        ];

        let points2 = [
            Vec2::new(points[0].y, points[0].x),
            Vec2::new(points[1].y, points[1].x),
            Vec2::new(points[4].y, points[4].x),
            Vec2::new(points[3].y, points[3].x),
            Vec2::new(points[2].y, points[2].x),
        ];

        let magn2 = [
            Vec2::new(magn[0].y, magn[0].x),
            Vec2::new(magn[1].y, magn[1].x),
            Vec2::new(magn[4].y, magn[4].x),
            Vec2::new(magn[3].y, magn[3].x),
            Vec2::new(magn[2].y, magn[2].x),
        ];

        for i in 0..cols {
            for j in 0..rows {
                for k in 0..5 {
                    let mut elem = if i % 2 == j % 2 {
                        (points[k], magn[k])
                    } else {
                        (points2[k], magn2[k])
                    };

                    if elem.1.y.is_sign_negative() {
                        elem.1 *= -1.0;
                    } else if elem.1.y.is_zero() && elem.1.x.is_sign_negative() {
                        elem.1.x *= -1.0;
                    }

                    elem.0 += Vec2::new(i as f64, j as f64) * 816.0;

                    elements.push(Element {
                        pos: elem.0,
                        magn: elem.1,
                    });
                }
            }
        }

        System::new(elements)
    }
}
