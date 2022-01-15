use crate::backend::register_allocation::RegisterInterface;

use super::Graph;

// For simplify the ranges are split into a low and high set
// low is unsorted and elements are picked as though they are from a stack
// high is sorted from highest degree to lowest degree
// This allows easy draining of all elements that get a lower degree by selection
// The precolored nodes are filter out as they cannot be spilled. Their degree is updated
pub fn simplify<R: RegisterInterface>(graph: &mut Graph<R>) -> Vec<u32> {
    let reg_count = R::REG_COUNT as u32;
    let mut stack = Vec::with_capacity(graph.length);
    let (mut low, mut high): (Vec<_>, _) = (0..graph.live_ranges.len())
        .filter(|&i| graph.live_ranges[i].precolor.is_none())
        .filter(|&i| !graph.live_ranges[i].vregs.is_empty())
        .partition(|&i| graph.degree[i] < reg_count);

    high.sort_unstable_by_key(|&i| u32::MAX - graph.degree[i]);

    log::trace!("low:{:?}\nhigh:{:?}", low, high);

    loop {
        while low.len() > 0 {
            // Choose one low member and remove it
            let m = low.pop().unwrap();
            stack.push(m as u32);

            // Update the degree of all neighbors
            for &n in &graph.adjacency_list[m] {
                graph.degree[n as usize] -= 1;
            }

            // Update high to account for the degree chance
            high.sort_unstable_by_key(|&i| u32::MAX - graph.degree[i]);
            log::trace!("high: {:?}", high);
            let start = high
                .iter()
                .enumerate()
                .find(|(_, &i)| graph.degree[i] < reg_count);
            if let Some((start, _)) = start {
                low.extend(high.drain(start..high.len()))
            }
        }
        if high.len() == 0 {
            break;
        }

        // Select a member of high with the lowest cost/degree
        // Fold is necessary because f32 does not implement ord
        // We get the index via enumerate to later delete this member
        // This can be done more efficiently, but is not necessary now
        let (index, _) = high.iter().map(|&i| graph.cost(i)).enumerate().fold(
            (0, f32::NEG_INFINITY),
            |(min_i, min), (i, cost)| {
                if cost < min {
                    (i, cost)
                } else {
                    (min_i, min)
                }
            },
        );
        let m = high[index];
        low.push(m);
        high.remove(index);
    }
    stack
}

impl<R: RegisterInterface> Graph<R> {
    fn cost(&self, i: usize) -> f32 {
        self.live_ranges[i].spill_cost / (self.degree[i] as f32)
    }
}
