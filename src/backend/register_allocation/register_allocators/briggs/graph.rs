use std::{collections::HashSet, fmt::Debug, iter::FromIterator};

use bitvec::prelude::BitVec;
use smallvec::SmallVec;

use crate::backend::register_allocation::RegisterInterface;

use super::LiveRange;

pub struct Graph<R: RegisterInterface> {
    pub bit_matrix: BitMatrix,
    pub live_ranges: Vec<LiveRange<R>>,
    pub vreg2live: Vec<Option<u32>>,
    pub adjacency_list: Vec<SmallVec<[u32; 4]>>,
    pub degree: Vec<u32>,
    pub length: usize,
}

impl<R: RegisterInterface> Graph<R> {
    pub fn new(
        live_ranges: Vec<LiveRange<R>>,
        vreg2live: Vec<Option<u32>>,
        length: usize,
    ) -> Graph<R> {
        let mut bit_matrix = BitMatrix::new(length);
        let adjacency_list = Vec::new();
        let mut degree = vec![0; length];

        for i in 0..R::REG_COUNT {
            degree[i] = (R::REG_COUNT - 1) as u32;
        }

        for index in 0..((R::REG_COUNT * (R::REG_COUNT - 1)) / 2) {
            bit_matrix.vector.set(index, true)
        }

        Graph {
            bit_matrix,
            live_ranges,
            vreg2live,
            adjacency_list,
            degree,
            length,
        }
    }

    pub fn adjust_spill_cost(&mut self, live_range: u32, loop_cost: f32) {
        self.live_ranges[live_range as usize].spill_cost += loop_cost;
    }

    pub fn degree(&self, index: u32) -> u32 {
        self.degree[index as usize]
    }

    pub fn merged_neighbors<T: FromIterator<u32>>(&self, i: u32, j: u32) -> T {
        let i = self.adjacency_list[i as usize].iter();
        let j = self.adjacency_list[j as usize].iter();
        let set: HashSet<_> = i.chain(j).cloned().collect();
        set.into_iter().collect::<T>()
    }

    pub fn drop_bit_matrix(&mut self) {
        self.bit_matrix = BitMatrix::new(0);
    }

    pub fn let_interfere(&mut self, x: u32, y: u32) {
        if cfg!(debug) {
            assert!(!self.bit_matrix.get(x, y))
        }

        if !self.bit_matrix.replace(x, y, true) {
            self.degree[x as usize] += 1;
            self.degree[y as usize] += 1;
        }
    }
    pub fn interfere(&self, x: u32, y: u32) -> bool {
        if cfg!(debug) {
            let xu = x as usize;
            let yu = y as usize;
            assert_eq!(
                self.adjacency_list[xu].contains(&y),
                self.adjacency_list[yu].contains(&x)
            )
        }
        let x = x as usize;
        self.adjacency_list[x].contains(&y)
    }

    pub fn fill_adjacency_list(&mut self) {
        self.adjacency_list = Vec::with_capacity(self.length);
        for index in 0..self.length {
            let size = self.degree[index] as usize;
            self.adjacency_list.push(SmallVec::with_capacity(size));
        }

        let mut iter = self.bit_matrix.vector.iter();
        for i in 0..self.length {
            for j in 0..i {
                if *iter.next().unwrap() {
                    self.adjacency_list[i].push(j as u32);
                    self.adjacency_list[j].push(i as u32);
                }
            }
        }
    }
}

impl<R: RegisterInterface> Debug for Graph<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "bitmatrix:{:?}", self.bit_matrix)?;
        writeln!(f, "live_ranges:")?;
        for live_range in &self.live_ranges {
            writeln!(f, "\t{:?}", live_range)?;
        }
        writeln!(f, "vreg2live:")?;
        for (vreg, live) in self.vreg2live.iter().enumerate() {
            if let Some(live) = live {
                writeln!(f, "\t{} => {}", vreg, live)?;
            }
        }
        writeln!(f, "adjecency lists:")?;
        for (live_range, adjecenies) in self.adjacency_list.iter().enumerate() {
            writeln!(f, "\t{} => {:?}", live_range, adjecenies)?;
        }
        writeln!(f, "degree: {:?}", self.degree)?;
        write!(f, "length: {}", self.length)?;
        Ok(())
    }
}

pub struct BitMatrix {
    vector: BitVec,
    size: usize,
}

impl BitMatrix {
    pub fn new(size: usize) -> BitMatrix {
        let len = (size * size + 1) / 2;
        BitMatrix {
            vector: BitVec::repeat(false, len),
            size: size,
        }
    }
    fn to_index(x: u32, y: u32) -> usize {
        assert_ne!(x, y);
        let i = std::cmp::min(x, y) as usize;
        let j = std::cmp::max(x, y) as usize;
        (j * (j - 1)) / 2 + i
    }
    pub fn _set(&mut self, x: u32, y: u32, value: bool) {
        let index = BitMatrix::to_index(x, y);
        self.vector.set(index, value);
    }
    pub fn get(&self, x: u32, y: u32) -> bool {
        let index = BitMatrix::to_index(x, y);
        *self.vector.get(index).unwrap()
    }
    pub fn replace(&mut self, x: u32, y: u32, value: bool) -> bool {
        let index = BitMatrix::to_index(x, y);
        self.vector.replace(index, value)
    }
}

impl Debug for BitMatrix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "")?;
        for i in 1..self.size {
            write!(f, "{:4}:[", i)?;
            for j in 0..i {
                write!(f, "{}", self.get(i as u32, j as u32) as u32)?;
            }
            writeln!(f, "]")?;
        }
        Ok(())
    }
}
