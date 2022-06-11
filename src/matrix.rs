use std::ops::{Index, IndexMut};

#[derive(Clone)]
pub struct Matrix {
    memory: Vec<f64>,
    size: usize,
}

impl Matrix {
    #[inline(always)]
    pub fn new(size: usize) -> Self {
        let memory = std::iter::repeat(0.0).take(size * size).collect();
        Self { memory, size }
    }

    #[inline(always)]
    pub fn row(&self, row: usize) -> &[f64] {
        &self.memory[row * self.size..(row + 1) * self.size]
    }

    #[inline(always)]
    pub fn row_mut(&mut self, row: usize) -> &mut [f64] {
        &mut self.memory[row * self.size..(row + 1) * self.size]
    }

    #[inline(always)]
    pub unsafe fn get_unchecked(&self, (row, col): (usize, usize)) -> &f64 {
        self.memory.get_unchecked(row * self.size + col)
    }

    #[inline(always)]
    pub unsafe fn get_unchecked_mut(&mut self, (row, col): (usize, usize)) -> &mut f64 {
        self.memory.get_unchecked_mut(row * self.size + col)
    }
}

impl Index<(usize, usize)> for Matrix {
    type Output = f64;

    #[inline(always)]
    fn index(&self, index: (usize, usize)) -> &Self::Output {
        &self.memory[index.0 * self.size + index.1]
    }
}

impl IndexMut<(usize, usize)> for Matrix {
    #[inline(always)]
    fn index_mut(&mut self, index: (usize, usize)) -> &mut Self::Output {
        &mut self.memory[index.0 * self.size + index.1]
    }
}