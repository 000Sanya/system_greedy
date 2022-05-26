use crate::{Element, System};

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
