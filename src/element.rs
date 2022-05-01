use crate::system::Vec2;

#[derive(Clone, Debug)]
pub struct Element {
    pub pos: Vec2,
    pub magn: Vec2,
}

impl Element {
    pub fn new(pos: Vec2, magn: Vec2) -> Self {
        Self {
            pos, magn
        }
    }

    pub fn energy_with(&self, element: &Element) -> f64 {
        let pij = self.pos - element.pos;

        let mi = self.magn;
        let mj = element.magn;

        let r = pij.magnitude();
        let r3 = r * r * r;
        let r5 = r3 * r * r;

        let result = (mi.dot(mj) / r3) - 3.0 * ((mi.dot(pij) * mj.dot(pij)) / r5);

        if result.is_nan() {
            0.0
        } else {
            result
        }
    }
}
