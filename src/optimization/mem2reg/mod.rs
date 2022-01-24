mod promotable;

use std::collections::{HashMap, HashSet};

use promotable::find_promotable_variables;
use smallvec::SmallVec;

use crate::backend::ir::*;

use super::analysis::{
    self,
    live_variable::{live_variable, LiveVariableAnalysis},
    ControlFlowGraph,
};
use analysis::{find_vreg_use_count, DominatorTree};

pub fn mem2reg(function: &mut IRFunction) {
    let cfg = ControlFlowGraph::construct(&function.instructions);
    let vreg_use_count = find_vreg_use_count(function);
    let dominator_tree = DominatorTree::new(&cfg);

    let promotions = find_promotable_variables(
        &function.instructions,
        &vreg_use_count,
        &function.variables,
        &function.arguments,
    );
    log::info!("Promotable variables:{:?}", promotions);
    let live_variables = live_variable(&cfg, function, Some(&promotions));

    build_pruned_ssa(
        function,
        &cfg,
        &dominator_tree,
        &promotions,
        &live_variables,
    )
}

#[derive(Debug, Clone)]
struct Phi {
    src: SmallVec<[(u32, u32); 2]>,
    var: u32,
    dest: u32,
}

fn build_pruned_ssa(
    function: &mut IRFunction,
    cfg: &ControlFlowGraph,
    dom_tree: &DominatorTree,
    promotions: &HashSet<u32>,
    live_variables: &LiveVariableAnalysis,
) {
    let mut stack = vec![SmallVec::<[u32; 4]>::new(); function.variables.len()];
    let mut phi_list = vec![Vec::<Phi>::new(); cfg.len()];
    let mut vreg_mutations = HashMap::new();
    let mut count = function.vreg_count;

    let idf = DominatorTree::iterated_dominance_frontier(cfg, &dom_tree.immediate_dominator);
    for df in &idf {
        log::debug!("idf: {:?}", df);
    }

    for block in 0..cfg.len() {
        for var in live_variables.gen[block]
            .iter_ones()
            .filter(|&var| promotions.contains(&(var as u32)))
        {
            for dom in idf[block].iter_ones() {
                if live_variables.live_in[dom][var]
                    && !phi_list[dom].iter().any(|p| p.var == var as u32)
                {
                    phi_list[dom].push(Phi {
                        var: var as u32,
                        dest: 0,
                        src: SmallVec::new(),
                    })
                }
            }
        }
    }

    replace_mem(
        &mut stack,
        &mut phi_list,
        &mut vreg_mutations,
        &mut function.instructions,
        cfg,
        dom_tree,
        promotions,
        &mut count,
        0,
    );
    function.vreg_count = count;

    log::debug!("phi_list: {:?}", phi_list);
    log::debug!("vreg_mutations:{:?}", vreg_mutations);

    write_back(
        &mut function.instructions,
        cfg,
        &function.variables,
        &phi_list,
        &vreg_mutations,
    );

    log::debug!("ir:\n{}", function);
}

fn replace_mem(
    stack: &mut [SmallVec<[u32; 4]>],
    phi_list: &mut [Vec<Phi>],
    vreg_mutations: &mut HashMap<u32, u32>,
    instructions: &mut [IRInstruction],
    cfg: &ControlFlowGraph,
    dom_tree: &DominatorTree,
    promotions: &HashSet<u32>,

    count: &mut u32,
    block: u32,
) {
    let b = block as usize;
    let mut stack_size = vec![0; stack.len()];

    //let mut var_map = HashMap::new();
    for phi in &mut phi_list[b] {
        let vreg = *count;
        *count += 1;
        phi.dest = vreg;
        stack[phi.var as usize].push(vreg);
        stack_size[phi.var as usize] += 1;
        //var_map.insert(phi.var, vreg);
    }

    let range_1 = cfg[block].instructions.clone();
    let range_2 = range_1.clone().skip(1);
    use IRInstruction::{AddrL, Load, Store};
    for (this, next) in range_1.zip(range_2) {
        match (&instructions[this], &instructions[next]) {
            (&AddrL(_, a, var), &Store(_size, store, address))
                if promotions.contains(&(var as u32)) && a == address =>
            {
                instructions[this] = IRInstruction::Nop;
                instructions[next] = IRInstruction::Nop;
                stack[var].push(store);
                stack_size[var] += 1;
            }
            (&AddrL(_, a, var), &Load(_size, load, address))
                if promotions.contains(&(var as u32)) && a == address =>
            {
                instructions[this] = IRInstruction::Nop;
                instructions[next] = IRInstruction::Nop;
                let vreg = stack[var].last().cloned().unwrap_or(0);
                vreg_mutations.insert(load, vreg);
            }

            _ => (),
        }
    }

    for &succ in &cfg[b].successors {
        for phi in phi_list[succ as usize].iter_mut() {
            phi.src
                .push((block, stack[phi.var as usize].last().cloned().unwrap_or(0)));
        }
    }

    for &child in dom_tree.dominants[b].iter().filter(|&&d| d != block) {
        replace_mem(
            stack,
            phi_list,
            vreg_mutations,
            instructions,
            cfg,
            dom_tree,
            promotions,
            count,
            child,
        );
    }

    for var in 0..stack.len() {
        let len = stack[var].len() - stack_size[var];
        stack[var].truncate(len);
    }
}

fn write_back(
    instructions: &mut [IRInstruction],
    cfg: &ControlFlowGraph,
    variables: &[IRVariable],
    phi_list: &[Vec<Phi>],
    vreg_mutations: &HashMap<u32, u32>,
) {
    for block in cfg
        .iter()
        .filter(|&b| !phi_list[b.label as usize].is_empty())
    {
        let b = block.label as usize;

        use IRInstruction::Label;
        let phi: &mut IRPhi = &mut *match &mut instructions[block.instructions.start as usize] {
            Label(Some(phi), _) => phi,
            Label(phi, _) => {
                *phi = Some(IRPhi::empty(Vec::new()));
                phi.as_mut().unwrap()
            }
            _ => unreachable!(),
        };
        log::trace!("phi:{:?}", phi);

        for p in &phi_list[b] {
            if let Some((i, _)) = phi.targets.iter().enumerate().find(|(_, &t)| t == p.dest) {
                phi.sources[i].extend_from_slice(&p.src);
            } else {
                phi.targets.push(p.dest);
                phi.size.push(variables[p.var as usize].size);
                phi.sources.push(p.src.clone())
            }
        }
    }

    for instruction in instructions {
        let used = instruction.get_mut_used();
        for usage in used {
            while vreg_mutations.contains_key(&*usage) {
                let copy = *usage;
                *usage = vreg_mutations[&copy];
            }
        }
    }
}
