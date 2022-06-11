use std::f64::consts::PI;
use bitvec::prelude::BitVec;
use plotters::prelude::{Polygon, ShapeStyle};
use vek::{Mat2, Vec2};
use crate::{Element, System};

pub fn grey_bitvec(g: BitVec) -> BitVec {
    let mut g1 = g.clone();
    g1.shift_right(1);
    g ^ g1
}

pub fn get_part_from_system(system: &System, elements: &[usize]) -> Vec<Element> {
    let mut elements: Vec<_> = system
        .elements()
        .iter()
        .enumerate()
        .filter(|(i, _)| elements.contains(i))
        .map(|(_, e)| e)
        .copied()
        .collect();

    let min_x = elements.iter().min_by_key(|e| e.pos.x).unwrap();
    let min_y = elements.iter().min_by_key(|e| e.pos.y).unwrap();
    let offset = vek::Vec2::new(min_x.pos.x, min_y.pos.y);

    elements.iter_mut().for_each(|e| e.pos -= offset);

    elements
}

pub fn generate_arrow<S: Into<ShapeStyle>>(l: f64, h: f64, ah: f64, al: f64, dir: Vec2<f64>, scale: f64, style: S) -> Polygon<(i32, i32)> {
    let l2 = l - al;

    let angle = dir.y.atan2(dir.x);
    let angle = if angle < 0. {
        angle + 2. * PI
    } else {
        angle
    };
    let matrix = Mat2::identity().scaled_2d(Vec2::new(scale, scale)).rotated_z(angle);

    let points = [
        (-l, h), (-l, -h), (l2, -h), (l2, -ah), (l, 0.), (l2, ah), (l2, h)
    ]
        .map(Vec2::<f64>::from)
        .map(|x| matrix * x)
        .map(|v| (v.x as i32, v.y as i32));

    Polygon::new(points, style)
}