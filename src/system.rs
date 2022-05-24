use std::cmp::Reverse;
use crate::Element;
use bitvec::prelude::BitVec;
use ordered_float::OrderedFloat;
use std::fmt::Write;
use ndarray::Array2;
use umya_spreadsheet::OrientationValues::Default;

pub type Vec2 = vek::Vec2<f64>;

pub struct CellRef {
    col: u32,
    row: u32,
}

#[derive(Clone, Debug)]
pub struct System {
    elements: Vec<Element>,
    element_neighbors: Vec<Vec<(usize, OrderedFloat<f64>)>>,
    system_state: BitVec,
    energy_matrix: Vec<Vec<f64>>,
    energy_matrix_default: Vec<Vec<f64>>,
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

        let mut energy_matrix = Vec::with_capacity(elements.len());
        let mut row_energies = Vec::with_capacity(elements.len());
        for elem in &elements {
            let mut energies = Vec::with_capacity(elements.len());
            for e in &elements {
                energies.push(elem.energy_with(e))
            }
            row_energies.push(energies.iter().sum());
            energy_matrix.push(energies);
        }

        let energy_matrix_default = energy_matrix.clone();

        let system_state = BitVec::repeat(false, elements.len());

        let plus = system_state.count_ones();
        let minus = system_state.count_zeros();
        let spin_excess = plus as i32 - minus as i32;

        let energy = row_energies.iter().sum::<f64>();

        Self {
            elements,
            element_neighbors,
            system_state,
            energy_matrix,
            energy_matrix_default,
            row_energies,
            energy,
            spin_excess,
        }
    }

    pub fn from_default_matrix(default_matrix: Vec<Vec<f64>>) -> Self {
        let row_energies: Vec<_> = default_matrix.iter().map(|row| row.iter().sum()).collect();
        let system_state = BitVec::repeat(false, default_matrix.len());
        let plus = system_state.count_ones();
        let minus = system_state.count_zeros();
        let spin_excess = plus as i32 - minus as i32;
        let energy = row_energies.iter().sum::<f64>();
        let elements = (0..default_matrix.len()).map(|_| Element::new(Vec2::zero(), Vec2::zero())).collect();
        let element_neighbors = Vec::new();
        let energy_matrix = default_matrix.clone();
        let energy_matrix_default = default_matrix;

        Self {
            elements,
            element_neighbors,
            system_state,
            energy_matrix,
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
    pub fn system_size(&self) -> usize {
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
        self.recalculate_energy();
        self.recalculate_spin_excess();
    }

    #[inline(always)]
    pub fn row_energies(&self) -> &Vec<f64> {
        &self.row_energies
    }

    #[inline(always)]
    pub fn energy_matrix(&self) -> &Vec<Vec<f64>> { &self.energy_matrix }

    #[inline(always)]
    pub fn default_energy_matrix(&self) -> &Vec<Vec<f64>> { &self.energy_matrix_default }

    #[inline(always)]
    pub fn element_neighbors(&self) -> &Vec<Vec<(usize, OrderedFloat<f64>)>> {
        &self.element_neighbors
    }

    #[inline(always)]
    pub fn neighbors<'a>(&'a self, index: usize, radius: f64) -> impl Iterator<Item=(usize, OrderedFloat<f64>)> + 'a {
        self.element_neighbors[index]
            .iter()
            .copied()
            .take_while(move |(_, d)| d.0 <= radius)
    }

    #[inline(always)]
    pub fn neighbors2<'a>(&'a self, index: usize, radius: f64) -> impl Iterator<Item=(usize, f64)> + 'a {
        self.neighbors(index, radius)
            .map(|(i, d)| (i, d.0))
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
        for row in 0..self.energy_matrix_default.len() {
            for col in 0..self.energy_matrix_default[row].len() {
                self.energy_matrix[row][col] = self.energy_matrix_default[row][col]
                    * bool_to_one(self.system_state[row])
                    * bool_to_one(self.system_state[col])
            }
            self.row_energies[row] = self.energy_matrix[row].iter().sum::<f64>();
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
        match self.system_state[spin] {
            true => self.spin_excess += 2,
            false => self.spin_excess -= 2,
        }

        for i in 0..self.system_size() {
            if i != spin {
                self.row_energies[spin] -= 2.0 * self.energy_matrix[spin][i];
                self.row_energies[i] -= 2.0 * self.energy_matrix[spin][i];
                self.energy -= 4.0 * self.energy_matrix[spin][i];

                self.energy_matrix[spin][i] *= -1.0;
                self.energy_matrix[i][spin] *= -1.0;
            }
        }
    }

    pub fn set_spins(&mut self, spines: impl Iterator<Item=(usize, bool)>) {
        for (spin, state) in spines {
            self.set_spin(spin, state);
        }
    }

    pub fn reverse_spins(&mut self, spines: impl Iterator<Item=usize>) {
        for spin in spines {
            self.reverse_spin(spin)
        }
    }

    pub fn greedy_step(&mut self) {
        let index = self
            .row_energies
            .iter()
            .enumerate()
            .max_by_key(|(_, x)| OrderedFloat(**x))
            .unwrap().0;
        self.reverse_spin(index);
    }

    pub fn all_row_energies_negative(&self) -> bool {
        self.row_energies.iter().all(|x| x.is_sign_negative())
    }

    pub fn state_from_binary_string(&mut self, bin_str: &str) {
        let iter: Vec<_> = bin_str
            .chars()
            .filter(|c| *c == '1' || *c == '0')
            .map(|c| c == '1')
            .enumerate()
            .filter_map(|(i, value)| (self.system_state[i] != value).then(|| i))
            .collect();

        self.reverse_spins(iter.iter().copied());
    }

    pub fn print_matrix(&self) {
        use prettytable::{Cell, Row, Table};

        let mut table = Table::new();
        for row in &self.energy_matrix {
            table.add_row(Row::new(
                row.iter()
                    .map(|c| format!("{:.6}", c))
                    .map(|c| Cell::new(&c))
                    .collect(),
            ));
        }

        table.printstd();
    }

    pub fn stats(&self) {
        use prettytable::{Cell, Row, Table};
        use std::collections::HashMap;
        let mut map = HashMap::<_, (String, f64, f64)>::new();

        for energies in &self.energy_matrix {
            for energy in energies {
                let energy = (energy * 1_0000000000.0).round() / 1_0000000000.0;
                map.entry(OrderedFloat(energy.abs()))
                    .and_modify(|(s, sum, sum_abs)| {
                        s.push(if energy.is_sign_positive() { '+' } else { '-' });
                        *sum += energy;
                        *sum_abs += energy.abs();
                    })
                    .or_insert((String::new(), 0.0, 0.0));
            }
        }

        let mut values: Vec<_> = map.iter().collect();
        values.sort_unstable_by_key(|(f, _)| Reverse(**f));

        let mut table = Table::new();

        table.add_row(Row::new(vec![
            Cell::new("energy"),
            Cell::new("signs"),
            Cell::new("p"),
            Cell::new("m"),
            Cell::new("sum"),
            Cell::new("sum_abs"),
        ]));

        for (energy, (signs, sum, sum_abs)) in values.iter().take(10) {
            let p = signs.chars().filter(|c| *c == '+').count();
            let m = signs.chars().filter(|c| *c == '-').count();
            table.add_row(Row::new(vec![
                Cell::new(&energy.to_string()),
                Cell::new(&*signs),
                Cell::new(&p.to_string()),
                Cell::new(&m.to_string()),
                Cell::new(&sum.to_string()),
                Cell::new(&sum_abs.to_string()),
            ]));
        }

        table.printstd();
    }

    pub fn save_system(&self, filename: impl AsRef<std::path::Path>) {
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

    pub fn save_in_excel(&self, filename: impl AsRef<std::path::Path>, zero_if: f64) {
        use umya_spreadsheet::helper::coordinate::coordinate_from_index;
        use umya_spreadsheet::*;

        const START_INDEX_CELL: CellRef = CellRef { row: 1, col: 1 };
        const START_VALUE_CELL: CellRef = CellRef { row: 2, col: 1 };

        const START_MATRIX_INDEX: CellRef = CellRef { row: 5, col: 1 };
        const START_MATRIX_VALUE: CellRef = CellRef { row: 6, col: 2 };

        let size = self.system_state.len() as u32;

        let mut book = new_file();

        let mut full = Worksheet::default();
        full.set_title("Full");

        let mut diagonal = Worksheet::default();
        diagonal.set_title("Diagonal");

        full.get_cell_by_column_and_row_mut(&START_INDEX_CELL.col, &START_INDEX_CELL.row)
            .set_value("Index");
        diagonal
            .get_cell_by_column_and_row_mut(&START_INDEX_CELL.col, &START_INDEX_CELL.row)
            .set_value("Index");

        full.get_cell_by_column_and_row_mut(&START_VALUE_CELL.col, &START_VALUE_CELL.row)
            .set_value("Value");
        diagonal
            .get_cell_by_column_and_row_mut(&START_VALUE_CELL.col, &START_VALUE_CELL.row)
            .set_value("Value");

        for (i, v) in self.system_state.iter().enumerate() {
            let i = i as u32;

            full.get_cell_by_column_and_row_mut(&(START_INDEX_CELL.col + 1 + i), &START_INDEX_CELL.row)
                .set_value_from_u32(i as u32);
            diagonal
                .get_cell_by_column_and_row_mut(&(START_INDEX_CELL.col + 1 + i), &START_INDEX_CELL.row)
                .set_value_from_u32(i as u32);

            full.get_cell_by_column_and_row_mut(
                &(START_VALUE_CELL.col + 1 + i),
                &START_VALUE_CELL.row,
            )
                .set_value_from_i32(bool_to_one(*v) as i32);
            diagonal
                .get_cell_by_column_and_row_mut(
                    &(START_VALUE_CELL.col + 1 + i),
                    &START_VALUE_CELL.row,
                )
                .set_value_from_i32(bool_to_one(*v) as i32);

            full.get_cell_by_column_and_row_mut(
                &(START_MATRIX_INDEX.col + 1 + i),
                &START_MATRIX_INDEX.row,
            )
                .set_value_from_u32(i as u32);
            diagonal
                .get_cell_by_column_and_row_mut(
                    &(START_MATRIX_INDEX.col + 1 + i),
                    &START_MATRIX_INDEX.row,
                )
                .set_value_from_u32(i as u32);

            full.get_cell_by_column_and_row_mut(
                &START_MATRIX_INDEX.col,
                &(START_MATRIX_INDEX.row + 1 + i),
            )
                .set_value_from_u32(i as u32);
            diagonal
                .get_cell_by_column_and_row_mut(
                    &START_MATRIX_INDEX.col,
                    &(START_MATRIX_INDEX.row + 1 + i),
                )
                .set_value_from_u32(i as u32);
        }

        let sum_start_cell = CellRef {
            row: START_MATRIX_VALUE.row + size + 1,
            ..START_MATRIX_VALUE
        };

        for y in 0..size {
            let y = y as u32;
            let yi = START_MATRIX_VALUE.row + y;
            let y_ref =
                coordinate_from_index(&(START_VALUE_CELL.col + 1 + y), &START_VALUE_CELL.row);
            for x in 0..size {
                let x = x as u32;
                let xi = START_MATRIX_VALUE.col + x;
                let x_ref =
                    coordinate_from_index(&(START_VALUE_CELL.col + 1 + x), &START_VALUE_CELL.row);

                let e = self.energy_matrix_default[y as usize][x as usize];
                full.get_cell_by_column_and_row_mut(&xi, &yi)
                    .set_formula(format!(
                        "={y_ref} * {x_ref} * {}",
                        if e.abs() >= zero_if { e } else { 0.0 }
                    ));

                if x >= y {
                    diagonal
                        .get_cell_by_column_and_row_mut(&xi, &yi)
                        .set_formula(format!(
                            "={y_ref} * {x_ref} * {}",
                            if e.abs() >= zero_if { e } else { 0.0 }
                        ));
                }
            }

            let start_ref =
                coordinate_from_index(&(START_MATRIX_VALUE.col + y), &START_MATRIX_VALUE.row);
            let end_ref = coordinate_from_index(
                &(START_MATRIX_VALUE.col + y),
                &(START_MATRIX_VALUE.row + size - 1),
            );

            full.get_cell_by_column_and_row_mut(&(sum_start_cell.col + y), &sum_start_cell.row)
                .set_formula(format!("=SUM({start_ref}:{end_ref})"));
            diagonal
                .get_cell_by_column_and_row_mut(&(sum_start_cell.col + y), &sum_start_cell.row)
                .set_formula(format!("=SUM({start_ref}:{end_ref})"));
        }

        let start_ref = coordinate_from_index(&sum_start_cell.col, &sum_start_cell.row);
        let end_ref = coordinate_from_index(&(sum_start_cell.col + size - 1), &sum_start_cell.row);

        full.get_cell_by_column_and_row_mut(&(sum_start_cell.col + size + 1), &sum_start_cell.row)
            .set_formula(format!("=SUM({start_ref}:{end_ref}) / 2"));
        diagonal
            .get_cell_by_column_and_row_mut(&(sum_start_cell.col + size + 1), &sum_start_cell.row)
            .set_formula(format!("=SUM({start_ref}:{end_ref})"));

        *book.get_sheet_collection_mut() = Vec::with_capacity(2);
        book.add_sheet(full).expect("error");
        book.add_sheet(diagonal).expect("error");

        writer::xlsx::write(&book, filename.as_ref()).expect("Error");
    }
}

fn bool_to_one(b: bool) -> f64 {
    match b {
        true => 1.0,
        false => -1.0,
    }
}
