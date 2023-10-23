// This is a rust implementation of the example
// found in tskit-c

use std::collections::HashMap;

use anyhow::Result;
use clap::Parser;
#[cfg(test)]
use rand::distributions::Distribution;
use rand::prelude::*;
use rand::SeedableRng;

#[derive(Debug)]
struct Edge {
    left: tskit::Position,
    right: tskit::Position,
    child: tskit::NodeId,
    previous: Option<usize>,
}

#[derive(Default)]
struct EdgeBuffer {
    parent: Vec<tskit::NodeId>,
    last: HashMap<tskit::NodeId, usize>,
    edges: Vec<Edge>,
}

impl EdgeBuffer {
    fn buffer_edge(
        &mut self,
        left: tskit::Position,
        right: tskit::Position,
        parent: tskit::NodeId,
        child: tskit::NodeId,
    ) {
        if let Some(last) = self.last.get_mut(&parent) {
            self.edges.push(Edge {
                left,
                right,
                child,
                previous: Some(*last),
            });
            *last = self.edges.len() - 1;
        } else {
            self.edges.push(Edge {
                left,
                right,
                child,
                previous: None,
            });
            self.last.insert(parent, self.edges.len() - 1);
            self.parent.push(parent);
        }
    }

    fn clear(&mut self) {
        self.parent.clear();
        self.last.clear();
        self.edges.clear();
    }
}

fn rotate_edges(bookmark: &tskit::types::Bookmark, tables: &mut tskit::TableCollection) {
    let num_edges = tables.edges().num_rows().as_usize();
    let left =
        unsafe { std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.left, num_edges) };
    let right =
        unsafe { std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.right, num_edges) };
    let parent =
        unsafe { std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.parent, num_edges) };
    let child =
        unsafe { std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.child, num_edges) };
    let mid = bookmark.edges().as_usize();
    left.rotate_left(mid);
    right.rotate_left(mid);
    parent.rotate_left(mid);
    child.rotate_left(mid);
}

// ANCHOR: haploid_wright_fisher
fn simulate(
    seed: u64,
    popsize: usize,
    num_generations: i32,
    simplify_interval: i32,
) -> Result<tskit::TreeSequence> {
    if popsize == 0 {
        return Err(anyhow::Error::msg("popsize must be > 0"));
    }
    if num_generations == 0 {
        return Err(anyhow::Error::msg("num_generations must be > 0"));
    }
    if simplify_interval == 0 {
        return Err(anyhow::Error::msg("simplify_interval must be > 0"));
    }
    let mut tables = tskit::TableCollection::new(1.0)?;

    // create parental nodes
    let mut parents_and_children = {
        let mut temp = vec![];
        let parental_time = f64::from(num_generations);
        for _ in 0..popsize {
            let node = tables.add_node(0, parental_time, -1, -1)?;
            temp.push(node);
        }
        temp
    };

    // allocate space for offspring nodes
    parents_and_children.resize(2 * parents_and_children.len(), tskit::NodeId::NULL);

    // Construct non-overlapping mutable slices into our vector.
    let (mut parents, mut children) = parents_and_children.split_at_mut(popsize);

    let parent_picker = rand::distributions::Uniform::new(0, popsize);
    let breakpoint_generator = rand::distributions::Uniform::new(0.0, 1.0);
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
    let mut bookmark = tskit::types::Bookmark::default();

    let mut buffer = EdgeBuffer::default();
    for birth_time in (0..num_generations).rev() {
        for c in children.iter_mut() {
            let bt = f64::from(birth_time);
            let child = tables.add_node(0, bt, -1, -1)?;
            let left_parent = parents
                .get(parent_picker.sample(&mut rng))
                .ok_or_else(|| anyhow::Error::msg("invalid left_parent index"))?;
            let right_parent = parents
                .get(parent_picker.sample(&mut rng))
                .ok_or_else(|| anyhow::Error::msg("invalid right_parent index"))?;
            let breakpoint = breakpoint_generator.sample(&mut rng);
            buffer.buffer_edge(0_f64.into(), breakpoint.into(), *left_parent, child);
            buffer.buffer_edge(breakpoint.into(), 1_f64.into(), *right_parent, child);
            *c = child;
        }

        if birth_time % simplify_interval == 0 {
            for &parent in buffer.parent.iter().rev() {
                let mut last = buffer.last.get(&parent).cloned();
                while let Some(previous) = last {
                    let edge = &buffer.edges[previous];
                    tables.add_edge(edge.left, edge.right, parent, edge.child)?;
                    last = edge.previous;
                }
            }
            buffer.clear();
            rotate_edges(&bookmark, &mut tables);
            if let Some(idmap) =
                tables.simplify(children, tskit::SimplificationOptions::default(), true)?
            {
                // remap child nodes
                for o in children.iter_mut() {
                    *o = idmap[usize::try_from(*o)?];
                }
            }
            bookmark.set_edges(tables.edges().num_rows());
        }
        std::mem::swap(&mut parents, &mut children);
    }

    tables.build_index()?;
    let treeseq = tables.tree_sequence(tskit::TreeSequenceFlags::default())?;

    Ok(treeseq)
}
// ANCHOR_END: haploid_wright_fisher

#[derive(Clone, clap::Parser)]
struct SimParams {
    seed: u64,
    popsize: usize,
    num_generations: i32,
    simplify_interval: i32,
    treefile: Option<String>,
    #[clap(short, long, help = "Use bookmark to avoid sorting entire edge table.")]
    bookmark: bool,
}

fn main() -> Result<()> {
    let params = SimParams::parse();
    let treeseq = simulate(
        params.seed,
        params.popsize,
        params.num_generations,
        params.simplify_interval,
    )?;

    if let Some(treefile) = &params.treefile {
        treeseq.dump(treefile, 0)?;
    }

    Ok(())
}
