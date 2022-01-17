use std::collections::HashSet;

use crate::backend::register_allocation::RegisterInterface;

use super::Graph;

pub fn select<R: RegisterInterface>(
    graph: &mut Graph<R>,
    mut stack: Vec<u32>,
) -> Result<Vec<R>, HashSet<u32>> {
    log::debug!("Starting select phase");
    // Spill capacity pre-allocated. Inspired by arXiv:1412.7664
    let mut spill = HashSet::with_capacity(graph.length / 8);
    let mut color = vec![R::REG_DEFAULT; graph.length];

    let precolored = graph
        .live_ranges
        .iter()
        .enumerate()
        .filter_map(|(i, live_range)| Some(i).zip(live_range.precolor));

    for (i, precolor) in precolored {
        color[i] = precolor;
    }

    while let Some(node) = stack.pop() {
        let index = node as usize;
        let mut used_colors = vec![false; R::REG_COUNT];
        for &n in &graph.adjacency_list[index] {
            let color_index: usize = color[n as usize].into();
            used_colors[color_index] = true;
        }

        if let Some((found_color, _)) = used_colors.iter().enumerate().find(|&(_, &b)| !b) {
            color[index] = R::REG_LOOKUP[found_color];
        } else {
            for &vregs in &graph.live_ranges[index].vregs {
                spill.insert(vregs);
            }
        }
    }

    if spill.is_empty() {
        Ok(color)
    } else {
        Err(spill)
    }
}
