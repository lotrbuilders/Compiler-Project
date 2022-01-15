use std::fmt::Display;
use std::ops::{Index, Range};

use smallvec::{smallvec, SmallVec};

use crate::backend::register_allocation::RegisterInterface;

//use super::register_location::NOT_ALLOCATED;
use super::register_location::RegisterLocation;
use super::RegisterRange;

#[derive(Debug, Clone)]
pub struct RegisterAllocation<R: RegisterInterface> {
    pub locations: SmallVec<[RegisterRange<R>; 1]>,
}

#[allow(dead_code)]
impl<R: RegisterInterface> RegisterAllocation<R> {
    pub fn empty() -> RegisterAllocation<R> {
        RegisterAllocation {
            locations: SmallVec::new(),
        }
    }
    pub fn new(loc: R, start: u32) -> RegisterAllocation<R> {
        RegisterAllocation {
            locations: smallvec![RegisterRange::new(loc, start..start)],
        }
    }
    pub fn live_at(&self, index: u32) -> bool {
        match self.locations[index as usize].loc {
            None => false,
            _ => true,
        }
    }
    pub fn start(&mut self, loc: R, start: u32) {
        self.locations.push(RegisterRange::new(loc, start..start))
    }
    pub fn end(&mut self, end: u32) {
        let reg = self.locations.last_mut().unwrap();
        let start = reg.range.start;
        let end = end + 1;
        reg.range = start..end;
    }
    pub fn end_prev(&mut self, end: u32) {
        let reg = self.locations.last_mut().unwrap();
        let start = reg.range.start;
        reg.range = start..end;
    }

    pub fn range(loc: R, range: Range<u32>) -> RegisterAllocation<R> {
        RegisterAllocation {
            locations: smallvec![RegisterRange::new(loc, range)],
        }
    }

    pub fn insert(&mut self, reg: R, loc: u32) {
        for (i, location) in self.locations.iter().enumerate() {
            if location.range.contains(&loc) {
                let normal_reg = location.loc;
                let end = location.range.end;

                self.locations[i].range.end = loc;
                let during = RegisterRange {
                    range: loc..loc + 1,
                    loc: Some(reg),
                };
                let after = RegisterRange {
                    range: (loc + 1)..end,
                    loc: normal_reg,
                };
                self.locations.push(during);
                self.locations.push(after);
                return;
            }
        }
        unreachable!()
    }
}

impl<R: RegisterInterface> Index<u32> for RegisterAllocation<R> {
    type Output = Option<R>;
    fn index(&self, index: u32) -> &Self::Output {
        for range in &self.locations {
            if range.range.contains(&index) {
                return &range.loc;
            }
        }
        if let Some(range) = self.locations.last() {
            let start = range.range.start;
            let end = range.range.end;
            if start == end && index >= start {
                return &range.loc;
            }
        }
        &RegisterLocation::<R>::NOT_ALLOCATED
    }
}
impl<R: RegisterInterface> Index<usize> for RegisterAllocation<R> {
    type Output = Option<R>;
    fn index(&self, index: usize) -> &Self::Output {
        self.index(index as u32)
    }
}

impl<R: RegisterInterface + Display> Display for RegisterAllocation<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for entry in &self.locations {
            write!(
                f,
                "({}:{}..{}) ",
                entry.loc.as_ref().unwrap(),
                entry.range.start,
                entry.range.end
            )?;
        }
        write!(f, "]")
    }
}
