use crate::Element;
use bitvec::prelude::BitVec;
use ordered_float::OrderedFloat;
use std::cmp::Reverse;
use std::fmt::Write;
use std::io::BufRead;
use num_traits::Zero;
use tap::Tap;
use crate::matrix::Matrix;

pub type Vec2 = vek::Vec2<f64>;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Orientation {
    Down,
    Up,
}

#[derive(Clone)]
pub struct System {
    elements: Vec<Element>,
    element_neighbors: Vec<Vec<(usize, OrderedFloat<f64>)>>,
    system_state: BitVec,
    system_signs: Vec<i8>,
    energy_matrix_default: Matrix,
    row_energies: Vec<f64>,
    energy: f64,
    spin_excess: i32,
}

impl System {
    pub fn new(elements: Vec<Element>) -> Self {
        let mut element_neighbors = Vec::with_capacity(elements.len());

        for e1 in &elements {
            let mut neighbors = Vec::with_capacity(elements.len());

            for (i, e2) in elements.iter().enumerate() {
                let distance = e1.pos.distance(e2.pos);
                neighbors.push((i, distance));
            }

            neighbors.sort_by_key(|(_, d)| *d);
            element_neighbors.push(neighbors);
        }

        let size = elements.len();

        let mut energy_matrix_default = Matrix::new(size);
        let mut row_energies = Vec::with_capacity(elements.len());
        for (i, elem) in elements.iter().enumerate() {
            for (j, e) in elements.iter().enumerate() {
                energy_matrix_default[(i, j)] = elem.energy_with(e);
            }
            row_energies.push(energy_matrix_default.row(i).iter().sum());
        }

        let system_state = BitVec::repeat(false, elements.len());
        let system_signs = std::iter::repeat(1).take(size).collect();

        let plus = system_state.count_ones();
        let minus = system_state.count_zeros();
        let spin_excess = plus as i32 - minus as i32;

        let energy = row_energies.iter().sum::<f64>();

        Self {
            elements,
            element_neighbors,
            system_state,
            system_signs,
            energy_matrix_default,
            row_energies,
            energy,
            spin_excess,
        }
    }

    #[inline(always)]
    pub fn spin_excess(&self) -> i32 {
        self.spin_excess
    }

    #[inline(always)]
    pub fn size(&self) -> usize {
        self.elements.len()
    }

    #[inline(always)]
    pub fn energy(&self) -> f64 {
        self.energy / 2.0
    }

    #[inline(always)]
    pub fn system_state(&self) -> &BitVec {
        &self.system_state
    }

    #[inline(always)]
    pub fn set_system_state(&mut self, bits: BitVec) {
        assert_eq!(self.elements.len(), bits.len());
        self.system_state = bits;
        for (i, s) in self.system_state.iter().enumerate() {
            self.system_signs[i] =  if *s { 1 } else { -1 };
        }
        self.recalculate_energy();
        self.recalculate_spin_excess();
    }

    #[inline(always)]
    pub fn row_energies(&self) -> &[f64] {
        &self.row_energies
    }

    #[inline(always)]
    pub fn default_energy_matrix(&self) -> &Matrix {
        &self.energy_matrix_default
    }

    #[inline(always)]
    pub fn element_neighbors(&self) -> &Vec<Vec<(usize, OrderedFloat<f64>)>> {
        &self.element_neighbors
    }

    #[inline(always)]
    pub fn neighbors(
        &self,
        index: usize,
        radius: f64,
    ) -> impl Iterator<Item = (usize, OrderedFloat<f64>)> + '_ {
        self.element_neighbors[index]
            .iter()
            .copied()
            .take_while(move |(_, d)| d.0 <= radius)
    }

    #[inline(always)]
    pub fn neighbors2(&self, index: usize, radius: f64) -> impl Iterator<Item = (usize, f64)> + '_ {
        self.neighbors(index, radius).map(|(i, d)| (i, d.0))
    }

    #[inline(always)]
    pub fn max_radius(&self) -> f64 {
        self.element_neighbors.iter().flatten().map(|(_, r)| *r).max().unwrap().0
    }

    #[inline(always)]
    pub fn elements(&self) -> &Vec<Element> {
        &self.elements
    }

    pub fn recalculate_spin_excess(&mut self) {
        let plus = self.system_state.count_ones();
        let minus = self.system_state.count_zeros();

        self.spin_excess = plus as i32 - minus as i32;
    }

    pub fn recalculate_energy(&mut self) {
        let size = self.size();
        for row in 0..size {
            let mut row_energy = 0.0;
            for col in 0..size {
                row_energy += self.energy_matrix_default[(row, col)]
                    * self.system_signs[row] as f64
                    * self.system_signs[col] as f64
            }
            self.row_energies[row] = row_energy;
        }
        self.energy = self.row_energies.iter().sum::<f64>();
    }

    pub fn set_spin(&mut self, spin: usize, state: bool) {
        if self.system_state[spin] != state {
            self.reverse_spin(spin);
        }
    }

    pub fn reverse_spin(&mut self, spin: usize) {
        let new_spin = !self.system_state[spin];
        self.system_state.set(spin, new_spin);

        let new_sign = self.system_signs[spin] * -1;
        self.system_signs[spin] = new_sign;

        let mut size = self.size();
        let mut row_energy = self.row_energies[spin];
        let mut energy = self.energy;

        unsafe {
            for i in 0..size {
                let si = *self.system_signs.get_unchecked(i) as f64;
                let cell_energy = self.energy_matrix_default.get_unchecked((spin, i)) * 2.0 * new_sign as f64 * si;
                row_energy += cell_energy;
                *self.row_energies.get_unchecked_mut(i) += cell_energy;
                energy += 2.0 * cell_energy;
            }
        }

        self.row_energies[spin] = row_energy;
        self.energy = energy;
    }

    pub fn set_spins(&mut self, spines: impl Iterator<Item = (usize, bool)>) {
        for (spin, state) in spines {
            self.set_spin(spin, state);
        }
    }

    pub fn reverse_spins(&mut self, spines: impl Iterator<Item = usize>) {
        for spin in spines {
            self.reverse_spin(spin)
        }
    }

    pub fn spin_orientation(&self, spin: usize) -> Orientation {
        let direction = self.elements[spin].magn() * self.system_signs[spin] as f64;
        if direction.y.is_sign_positive() || direction.y.is_zero() && direction.x.is_sign_positive() {
            Orientation::Down
        } else {
            Orientation::Up
        }
    }

    pub fn set_spin_orientation(&mut self, spin: usize, orientation: Orientation) {
        if self.spin_orientation(spin) != orientation {
            self.reverse_spin(spin);
        }
    }

    pub fn load_mfsys(filename: impl AsRef<std::path::Path>) -> Self {
        let file = std::fs::File::open(filename).unwrap();
        let lines = std::io::BufReader::new(file).lines();

        let mut elements = Vec::new();
        let mut states = BitVec::new();
        for l in lines.skip_while(|l| l.as_ref().unwrap() != "[parts]").skip(1) {
            let line = l.unwrap();
            let parts: Vec<_> = line.split_whitespace().collect();
            if parts.len() < 8 {
                continue;
            }

            let x: f64 = parts[1].parse().unwrap();
            let y: f64 = parts[2].parse().unwrap();
            let state: i8 = parts[7].parse().unwrap();
            let mx: f64 = parts[4].parse::<f64>().unwrap() * if state == 1 { -1. } else { 1. };
            let my: f64 = parts[5].parse::<f64>().unwrap() * if state == 1 { -1. } else { 1. };

            elements.push(
                Element::new(Vec2::new(x, y), Vec2::new(mx, my))
            );

            states.push(state == 1);
        }

        System::new(elements)
            .tap_mut(|s| s.set_system_state(states))
    }

    pub fn save_mfsys(&self, filename: impl AsRef<std::path::Path>) {
        let mut buffer = String::new();
        writeln!(buffer, "[header]").expect("Error");
        writeln!(buffer, "dimensions=2").expect("Error");
        writeln!(buffer, "size={}", self.elements.len()).expect("Error");
        let state = self
            .system_state
            .iter()
            .map(|r| if *r { "1" } else { "0" })
            .fold(String::new(), |acc, part| acc + part);
        writeln!(buffer, "state={}", state).expect("Error");
        writeln!(buffer, "[parts]").expect("Error");
        for (id, row) in self.elements.iter().enumerate() {
            let state = if self.system_state[id] { "1" } else { "0" };
            let factor = bool_to_one(self.system_state[id]) * -1.0;
            writeln!(
                buffer,
                "{}\t{:.16}\t{:.16}\t{:.16}\t{:.16}\t{:.16}\t{:.16}\t{}",
                id,
                row.pos.x,
                row.pos.y,
                0.0,
                row.magn.x * factor,
                row.magn.y * factor,
                0.0,
                state
            )
            .expect("Error");
        }

        std::fs::write(filename, buffer).expect("Error on write to file");
    }
}

fn bool_to_one(b: bool) -> f64 {
    match b {
        true => 1.0,
        false => -1.0,
    }
}