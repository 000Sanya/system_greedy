use bitvec::prelude::BitVec;

pub fn grey_bitvec(g: BitVec) -> BitVec {
    let mut g1 = g.clone();
    g1.shift_right(1);
    g ^ g1
}