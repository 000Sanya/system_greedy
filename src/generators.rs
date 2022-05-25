use crate::system::Vec2;
use crate::{Element, System};
use num_traits::Zero;
use std::f64::consts::PI;

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

                    elements.push(Element::new(elem.0, elem.1));
                }
            }
        }

        System::new(elements)
    }

    pub fn honeycomb(rows: u64, cols: u64) -> System {
        let sqrt_3 = 3.0f64.sqrt();
        let skip_if_down = [10, 11, 15];
        let skip_if_right = 18;

        let mut elements = Vec::new();
        for row in 0..rows {
            for col in 0..cols {
                let pos = [
                    Vec2::new(sqrt_3, 0.75),
                    Vec2::new((3.0 * sqrt_3) / 4.0, 1.5),
                    Vec2::new(sqrt_3 / 4.0, 1.5),
                    Vec2::new(0.0, 0.75),
                    Vec2::new(sqrt_3 / 4.0, 0.0),
                    Vec2::new((3.0 * sqrt_3) / 4.0, 0.0),
                    Vec2::new((7.0 * sqrt_3) / 4.0, 1.5),
                    Vec2::new((5.0 * sqrt_3) / 4.0, 1.5),
                    Vec2::new((5.0 * sqrt_3) / 4.0, 0.0),
                    Vec2::new((7.0 * sqrt_3) / 4.0, 0.0),
                    Vec2::new(sqrt_3 / 2.0, 2.25),
                    Vec2::new((3.0 * sqrt_3) / 2.0, 2.25),
                    Vec2::new(sqrt_3 * 2.0, 0.75),
                    Vec2::new((9.0 * sqrt_3) / 4.0, 0.0),
                    Vec2::new((9.0 * sqrt_3) / 4.0, 1.5),
                    Vec2::new((5.0 * sqrt_3) / 2.0, 2.25),
                    Vec2::new((11.0 * sqrt_3) / 4.0, 0.0),
                    Vec2::new((11.0 * sqrt_3) / 4.0, 1.5),
                    Vec2::new(sqrt_3 * 3.0, 0.75),
                ];

                let magn = [
                    Vec2::new(0.0, 1.0),
                    Vec2::new(-sqrt_3 / 2.0, 0.5),
                    Vec2::new(sqrt_3 / 2.0, 0.5),
                    Vec2::new(0.0, 1.0),
                    Vec2::new(-sqrt_3 / 2.0, 0.5),
                    Vec2::new(sqrt_3 / 2.0, 0.5),
                    Vec2::new(-sqrt_3 / 2.0, 0.5),
                    Vec2::new(sqrt_3 / 2.0, 0.5),
                    Vec2::new(-sqrt_3 / 2.0, 0.5),
                    Vec2::new(sqrt_3 / 2.0, 0.5),
                    Vec2::new(0.0, 1.0),
                    Vec2::new(0.0, 1.0),
                    Vec2::new(0.0, 1.0),
                    Vec2::new(-sqrt_3 / 2.0, 0.5),
                    Vec2::new(sqrt_3 / 2.0, 0.5),
                    Vec2::new(0.0, 1.0),
                    Vec2::new(sqrt_3 / 2.0, 0.5),
                    Vec2::new(-sqrt_3 / 2.0, 0.5),
                    Vec2::new(0.0, 1.0),
                ];

                let down = row == rows - 1;
                let right = col == cols - 1;

                elements.extend(
                    (0..=18)
                        .filter(|x| !(down && skip_if_down.contains(x)))
                        .filter(|x| *x != skip_if_right || right)
                        .map(|i| {
                            Element::new(
                                pos[i]
                                    + Vec2::new(0.0 + sqrt_3 * 3.0 * col as f64, 3.0 * row as f64),
                                magn[i],
                            )
                        }),
                )
            }
        }

        System::new(elements)
    }

    pub fn trimer(a: f64, b: f64, rows: usize, cols: usize) -> System {
        let phi_1 = 60.0 * PI / 180.0;
        let phi_2 = 180.0 * PI / 180.0;
        let phi_3 = 300.0 * PI / 180.0;
        let l = 300.0;

        let tr = [
            (
                Vec2::new(a * phi_1.cos(), a * phi_1.sin()),
                Vec2::new(l * phi_1.cos(), l * phi_1.sin()),
            ),
            (
                Vec2::new(a * phi_2.cos(), a * phi_2.sin()),
                Vec2::new(l * phi_2.cos(), l * phi_2.sin()),
            ),
            (
                Vec2::new(a * phi_3.cos(), a * phi_3.sin()),
                Vec2::new(l * phi_3.cos(), l * phi_3.sin()),
            ),
        ];

        let dx = b;
        let dy = b * phi_1.sin();
        let offset = b * phi_1.cos();

        let mut elements = Vec::with_capacity(rows * cols * 3);

        for row in 0..rows {
            for col in 0..cols {
                if row % 2 == 0 {
                    elements.extend(tr.iter().map(|(p, m)| {
                        Element::new(p + Vec2::new(dx * col as f64, dy * row as f64), *m)
                    }))
                } else {
                    elements.extend(tr.iter().map(|(p, m)| {
                        Element::new(p + Vec2::new(offset + dx * col as f64, dy * row as f64), *m)
                    }))
                }
            }
        }

        System::new(elements)
    }

    pub fn wtf(a: f64, rows: usize, cols: usize) -> System {
        let alpha = 127. * PI / 180.;
        let betta = 53. * PI / 180.;
        let l = 450.;

        let dx = a * ((PI - alpha) / 2.).cos();
        let dy = a * ((PI - betta) / 2.).cos();
        let offset = (a.powi(2) + (a / 2.).powi(2)).sqrt();

        let tr = [
            (
                Vec2::new(-dx / 2., -dy / 2.),
                Vec2::new(l * (betta / 2.).cos(), l * (betta / 2.).sin()),
            ),
            (
                Vec2::new(-dx / 2., dy / 2.),
                Vec2::new(l * (betta / 2.).cos(), l * (-betta / 2.).sin()),
            ),
            (
                Vec2::new(dx / 2., -dy / 2.),
                Vec2::new(l * (betta / 2.).cos(), l * (-betta / 2.).sin()),
            ),
            (
                Vec2::new(dx / 2., dy / 2.),
                Vec2::new(l * (betta / 2.).cos(), l * (betta / 2.).sin()),
            ),
        ];

        let mut elements = Vec::with_capacity(rows * cols * 4);

        for row in 0..rows {
            for col in 0..cols {
                if (row + col) % 2 == 0 {
                    elements.extend(tr.iter().map(|(p, m)| {
                        Element::new(p + Vec2::new(col as f64 * offset, row as f64 * offset), *m)
                    }))
                } else {
                    elements.extend(tr.iter().map(|(p, m)| {
                        let mut p = *p;
                        let mut m = *m;
                        p.reverse();
                        m.reverse();
                        Element::new(p + Vec2::new(col as f64 * offset, row as f64 * offset), m)
                    }))
                }
            }
        }

        System::new(elements)
    }
}
