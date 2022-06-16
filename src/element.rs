use crate::system::Vec2;
use ordered_float::OrderedFloat;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Element {
    pub pos: vek::Vec2<OrderedFloat<f64>>,
    pub magn: vek::Vec2<OrderedFloat<f64>>,
}

impl Element {
    pub fn new(pos: Vec2, magn: Vec2) -> Self {
        Self {
            pos: pos.map(OrderedFloat),
            magn: magn.map(OrderedFloat),
        }
    }

    #[inline(always)]
    pub fn magn(&self) -> Vec2 {
        self.magn.map(|x| x.0)
    }

    #[inline(always)]
    pub fn energy_with(&self, element: &Element) -> f64 {
        let pij = self.pos.map(|x| x.0) - element.pos.map(|x| x.0);

        let mi = self.magn.map(|x| x.0);
        let mj = element.magn.map(|x| x.0);

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
