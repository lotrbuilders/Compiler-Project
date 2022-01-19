use super::ControlFlowGraph;
use smallvec::SmallVec;
use std::collections::HashSet;

pub struct DominatorTree {
    pub immediate_dominator: Vec<u32>,
    pub dominants: Vec<SmallVec<[u32; 4]>>,
    pub dominance_frontier: Vec<SmallVec<[u32; 4]>>,
}

impl DominatorTree {
    // To create the dominator tree we create a post order
    // That post order is used to efficiently index into the list of immidiate dominators
    // Most importantly the lookup of predecessors needs to map from forward index to reverse post index
    // This is the translated back
    pub fn new(cfg: &ControlFlowGraph) -> DominatorTree {
        let immediate_dominator = DominatorTree::find_immediate_dominators(cfg);

        let dominants = DominatorTree::find_dominants(cfg, &immediate_dominator);
        log::debug!("translated dominants:{:?}", dominants);

        let dominance_frontier = DominatorTree::find_dominance_frontier(cfg, &immediate_dominator);
        log::debug!("Dominance frontier:{:?}", dominance_frontier);

        DominatorTree {
            immediate_dominator,
            dominants,
            dominance_frontier,
        }
    }

    fn find_immediate_dominators(cfg: &ControlFlowGraph) -> Vec<u32> {
        let mut doms = vec![None; cfg.len()];
        let mut changed = true;
        let post_order = cfg.rev_post();
        let mut look_up = vec![0; cfg.len()];
        for (i, &b) in post_order.iter().enumerate() {
            look_up[b as usize] = i;
        }
        doms[look_up[0]] = Some(look_up[0] as u32);
        log::trace!("post_order:{:?}", post_order);
        log::trace!("look_up:{:?}", look_up);
        while changed {
            changed = false;
            log::debug!("new iteration: {:?}", doms);

            for (i, &block) in post_order.iter().enumerate().filter(|(_, &b)| b != 0) {
                let node = &cfg[block];

                let pred = node
                    .predecessors
                    .iter()
                    .map(|&p| look_up[p as usize])
                    .find(|&p| doms[p].is_some());

                let pred = if pred.is_some() {
                    pred.unwrap()
                } else {
                    continue;
                };

                let mut new_idom = pred; //look_up[pred as usize];

                for p in node
                    .predecessors
                    .iter()
                    .map(|&p| look_up[p as usize])
                    .filter(|&p| p != pred)
                {
                    if doms[p].is_some() {
                        new_idom = DominatorTree::intersect(new_idom, p, &doms);
                    }
                }
                if doms[i] != Some(new_idom as u32) {
                    doms[i] = Some(new_idom as u32);
                    changed = true;
                }
            }
        }
        log::debug!("untranslated dominator: {:?}", doms);
        let mut idom = vec![0; cfg.len()];
        for (i, &b) in post_order.iter().enumerate() {
            idom[b as usize] = post_order[doms[i].unwrap() as usize];
        }
        log::debug!("translated dominator:{:?}", idom);
        idom
    }

    fn intersect(b1: usize, b2: usize, doms: &[Option<u32>]) -> usize {
        let mut f1 = b1;
        let mut f2 = b2;
        while f1 != f2 {
            while f1 < f2 {
                f1 = doms[f1].unwrap() as usize;
            }
            while f2 < f1 {
                f2 = doms[f2].unwrap() as usize;
            }
        }
        f1
    }

    fn find_dominants(cfg: &ControlFlowGraph, idom: &Vec<u32>) -> Vec<SmallVec<[u32; 4]>> {
        let mut doms = vec![SmallVec::new(); cfg.len()];
        for (i, &dom) in idom.iter().enumerate() {
            doms[dom as usize].push(i as u32);
        }
        for i in 1..doms.len() {
            doms[i].push(i as u32);
        }
        doms
    }

    fn find_dominance_frontier(
        cfg: &ControlFlowGraph,
        immediate_dominator: &[u32],
    ) -> Vec<SmallVec<[u32; 4]>> {
        let idom = immediate_dominator;
        let mut df = vec![SmallVec::new(); cfg.len()];

        for block in cfg.iter().filter(|&b| b.predecessors.len() > 1) {
            for &pred in &block.predecessors {
                let mut temp = pred;
                while temp != idom[pred as usize] {
                    df[temp as usize].push(block.label);
                    temp = idom[temp as usize];
                }
            }
        }

        df
    }
}

impl<'a> ControlFlowGraph {
    fn rev_post(&'a self) -> Vec<u32> {
        let mut visited = HashSet::new();
        let mut list = Vec::with_capacity(self.len());

        self.reverse_post_order(&mut list, &mut visited, 0);
        list
    }

    fn reverse_post_order(&'a self, list: &mut Vec<u32>, visited: &mut HashSet<u32>, node: u32) {
        visited.insert(node);
        for &suc in &self[node].successors {
            if !visited.contains(&suc) {
                self.reverse_post_order(list, visited, suc);
            }
        }
        list.push(node)
    }
}
