use super::{IRLabel, IRReg, IRSize};

#[derive(Clone, Debug, PartialEq)]
pub struct IRPhi {
    pub targets: Vec<IRReg>,
    pub size: Vec<IRSize>,
    pub locations: Vec<IRLabel>,
    pub sources: Vec<Vec<IRReg>>,
}

impl IRPhi {
    pub fn empty(locations: Vec<IRLabel>) -> Box<IRPhi> {
        let len = locations.len();
        Box::new(IRPhi {
            targets: Vec::new(),
            size: Vec::new(),
            locations,
            sources: vec![Vec::new(); len],
        })
    }
    pub fn ternary(
        size: IRSize,
        locations: (IRLabel, IRLabel),
        result: u32,
        vreg: (u32, u32),
    ) -> Box<IRPhi> {
        let (l1, l2) = locations;
        let (v1, v2) = vreg;
        Box::new(IRPhi {
            targets: vec![result],
            size: vec![size],
            locations: vec![l1, l2],
            sources: vec![vec![v1], vec![v2]],
        })
    }
}
