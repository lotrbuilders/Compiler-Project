use std::collections::HashSet;

use smallvec::SmallVec;

use crate::backend::register_allocation::RegisterInterface;

use super::renumber::VregCopy;
use super::Graph;

#[derive(Debug, Clone)]
pub struct CoalesceSettings {
    pub conservative: bool,
    pub coalesce_split: bool,
    pub coalesce_argument: bool,
}

impl<R: RegisterInterface> Graph<R> {
    pub fn coalesce(
        &mut self,
        copies: &mut [SmallVec<[VregCopy; 2]>],
        settings: CoalesceSettings,
    ) -> bool {
        let mut modified = false;
        for instruction in copies {
            for copy in instruction {
                if settings.conservative {
                    modified |= self.conservative_coalesce(copy, &settings)
                } else {
                    modified |= self.liberal_coalesce(copy, &settings)
                }
            }
        }
        modified
    }

    fn conservative_coalesce(&mut self, copy: &mut VregCopy, settings: &CoalesceSettings) -> bool {
        match copy {
            VregCopy::ArgumentCopy { reg, .. }
                if self.significant_neighbors(copy) < R::REG_COUNT
                    && settings.coalesce_argument =>
            {
                let _ = reg;
                self.coalesce_copy(copy)
            }

            VregCopy::TargetBefore { reg, .. } | VregCopy::TargetAfter { reg, .. }
                if self.significant_neighbors(copy) < R::REG_COUNT =>
            {
                let _ = reg;
                self.coalesce_copy(copy)
            }

            VregCopy::PhiCopy { .. } | VregCopy::TwoAddress { .. }
                if self.significant_neighbors(copy) < R::REG_COUNT =>
            {
                self.coalesce_copy(copy)
            }
            _ => false,
        }
    }

    fn liberal_coalesce(&mut self, copy: &mut VregCopy, settings: &CoalesceSettings) -> bool {
        match copy {
            VregCopy::ArgumentCopy { .. } if settings.coalesce_argument => self.coalesce_copy(copy),
            VregCopy::TargetBefore { .. }
            | VregCopy::TargetAfter { .. }
            | VregCopy::PhiCopy { .. }
            | VregCopy::TwoAddress { .. } => self.coalesce_copy(copy),
            _ => false,
        }
    }

    fn significant_neighbors(&self, copy: &VregCopy) -> usize {
        let (i, j) = copy.destination::<R>(&self.vreg2live);
        //let i = self.vreg2live[i as usize].unwrap();
        //let j = self.vreg2live[j as usize].unwrap();
        let neighbors: HashSet<_> = self.merged_neighbors(i, j);
        neighbors
            .into_iter()
            .filter(|&n| self.degree(n) >= R::REG_COUNT as u32)
            .count()
    }

    fn coalesce_copy(&mut self, copy: &mut VregCopy) -> bool {
        let (destination, source) = copy.destination::<R>(&self.vreg2live);
        if destination == source {
            *copy = VregCopy::Coalesced;
            return true;
        }
        let (_dst, src) = (destination as usize, source as usize);
        if self.interfere(destination, source) {
            return false;
        }

        let mut neighbors: SmallVec<_> = self.merged_neighbors(destination, source);
        if cfg!(debug_assertions) {
            neighbors.sort_unstable();
        }
        let degree = neighbors.len() as u32;

        // neighbor in destination and source   -> remove src
        // neighbor in source only              -> replace src with dst
        let source_neigbors = self.adjacency_list[src].clone();
        for &n in &source_neigbors {
            if self.adjacency_list[n as usize].contains(&destination) {
                self.adjacency_list[n as usize].retain(|i| *i != source);
                self.degree[n as usize] -= 1;
            } else {
                self.adjacency_list[n as usize]
                    .iter_mut()
                    .filter(|i| **i == source)
                    .for_each(|x| *x = destination)
            }
        }

        let source_vregs = std::mem::replace(
            &mut self.live_ranges[source as usize].vregs,
            SmallVec::new(),
        );
        for vreg in source_vregs {
            self.vreg2live[vreg as usize] = Some(destination);
            self.live_ranges[destination as usize].vregs.push(vreg);
        }

        self.live_ranges[destination as usize].spill_cost +=
            self.live_ranges[source as usize].spill_cost;
        self.live_ranges[source as usize].spill_cost = 0.;

        self.degree[source as usize] = 0;
        self.degree[destination as usize] = degree;

        self.adjacency_list[source as usize] = SmallVec::new();
        self.adjacency_list[destination as usize] = neighbors;
        *copy = VregCopy::Coalesced;

        true
    }
}
