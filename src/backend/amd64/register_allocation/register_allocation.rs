use std::fmt::Display;
use std::ops::Index;

use super::super::Register;
use super::register_location::NOT_ALLOCATED;
use super::RegisterRange;

#[derive(Debug, Clone)]
pub struct RegisterAllocation {
    pub locations: Vec<RegisterRange>,
}

#[allow(dead_code)]
impl RegisterAllocation {
    pub fn empty() -> RegisterAllocation {
        RegisterAllocation {
            locations: Vec::new(),
        }
    }
    pub fn new(loc: Register, start: u32) -> RegisterAllocation {
        RegisterAllocation {
            locations: vec![RegisterRange::new(loc, start..start)],
        }
    }
    pub fn live_at(&self, index: u32) -> bool {
        match self.locations[index as usize].loc {
            None => false,
            _ => true,
        }
    }
    pub fn start(&mut self, loc: Register, start: u32) {
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
}

impl Index<u32> for RegisterAllocation {
    type Output = Option<Register>;
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
        &NOT_ALLOCATED
    }
}
impl Index<usize> for RegisterAllocation {
    type Output = Option<Register>;
    fn index(&self, index: usize) -> &Self::Output {
        self.index(index as u32)
    }
}

impl Display for RegisterAllocation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[")?;
        for entry in &self.locations {
            write!(
                f,
                "({}:{}..{}) ",
                entry.loc.unwrap(),
                entry.range.start,
                entry.range.end
            )?;
        }
        write!(f, "]")
    }
}
