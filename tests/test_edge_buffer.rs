#![cfg(feature = "edgebuffer")]

use proptest::prelude::*;
use rand::distributions::Distribution;
use rand::SeedableRng;

use tskit::EdgeBuffer;
use tskit::TableCollection;
use tskit::TreeSequence;

fn overlapping_generations(seed: u64, pdeath: f64, simplify: i32) -> TreeSequence {
    let mut tables = TableCollection::new(1.0).unwrap();
    let mut buffer = EdgeBuffer::default();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let popsize = 10;

    let mut parents = vec![];

    for _ in 0..popsize {
        let node = tables.add_node(0, 10.0, -1, -1).unwrap();
        parents.push(node);
    }

    let death = rand::distributions::Uniform::new(0., 1.0);
    let parent_picker = rand::distributions::Uniform::new(0, popsize);

    for birth_time in (0..10).rev() {
        let mut replacements = vec![];
        for i in 0..parents.len() {
            if death.sample(&mut rng) <= pdeath {
                replacements.push(i);
            }
        }
        let mut births = vec![];

        for _ in 0..replacements.len() {
            let parent_index = parent_picker.sample(&mut rng);
            let parent = parents[parent_index];
            let child = tables.add_node(0, birth_time as f64, -1, -1).unwrap();
            births.push(child);
            buffer.setup_births(&[parent], &[child]).unwrap();
            buffer.record_birth(parent, child, 0., 1.).unwrap();
            buffer.finalize_births();
        }

        for (r, b) in replacements.iter().zip(births.iter()) {
            assert!(*r < parents.len());
            parents[*r] = *b;
        }
        if birth_time % simplify == 0 {
            buffer.pre_simplification(&mut tables).unwrap();
            //tables.full_sort(tskit::TableSortOptions::default()).unwrap();
            if let Some(idmap) = tables
                .simplify(&parents, tskit::SimplificationOptions::default(), true)
                .unwrap()
            {
                // remap child nodes
                for o in parents.iter_mut() {
                    *o = idmap[usize::try_from(*o).unwrap()];
                }
            }
            buffer.post_simplification(&parents, &mut tables).unwrap();
        }
    }

    tables.build_index().unwrap();

    tables.tree_sequence(0.into()).unwrap()
}

fn overlapping_generations_streaming_simplification(
    seed: u64,
    pdeath: f64,
    simplify: i32,
) -> TreeSequence {
    let mut tables = TableCollection::new(1.0).unwrap();
    let mut buffer = EdgeBuffer::default();
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let popsize = 10;

    let mut parents = vec![];

    for _ in 0..popsize {
        let node = tables.add_node(0, 10.0, -1, -1).unwrap();
        parents.push(node);
    }

    let death = rand::distributions::Uniform::new(0., 1.0);
    let parent_picker = rand::distributions::Uniform::new(0, popsize);
    let mut node_map: Vec<tskit::NodeId> = vec![];

    for birth_time in (0..10).rev() {
        println!("birth time {birth_time:?}");
        let mut replacements = vec![];
        for i in 0..parents.len() {
            if death.sample(&mut rng) <= pdeath {
                replacements.push(i);
            }
        }
        let mut births = vec![];

        for _ in 0..replacements.len() {
            let parent_index = parent_picker.sample(&mut rng);
            assert!(parent_index < parents.len());
            let parent = parents[parent_index];
            let child = tables.add_node(0, birth_time as f64, -1, -1).unwrap();
            births.push(child);
            buffer.buffer_birth(parent, child, 0., 1.).unwrap();
        }

        for (r, b) in replacements.iter().zip(births.iter()) {
            assert!(*r < parents.len());
            parents[*r] = *b;
        }
        if birth_time % simplify == 0 {
            println!("simplifying!");
            node_map.resize(tables.nodes().num_rows().as_usize(), tskit::NodeId::NULL);
            tskit::simplfify_from_buffer(
                &parents,
                tskit::SimplificationOptions::default(),
                &mut tables,
                &mut buffer,
                Some(&mut node_map),
            )
            .unwrap();
            println!("{parents:?}");
            for o in parents.iter_mut() {
                assert!(o.as_usize() < node_map.len());
                *o = node_map[usize::try_from(*o).unwrap()];
                assert!(!o.is_null());
            }
            println!("remapped {parents:?}");
            buffer.post_simplification(&parents, &mut tables).unwrap();
        }
    }
    tables.build_index().unwrap();
    tables.tree_sequence(0.into()).unwrap()
}

#[cfg(test)]
proptest! {
    #[test]
    fn test_edge_buffer_overlapping_generations(seed in any::<u64>(),
                                                pdeath in 0.05..1.0,
                                                simplify_interval in 1..100i32) {
        let _ = overlapping_generations(seed, pdeath, simplify_interval);
    }
}

#[cfg(test)]
proptest! {
    #[test]
    fn test_edge_buffer_overlapping_generations_streaming_simplification(seed in any::<u64>(),
                                                pdeath in 0.05..1.0,
                                                simplify_interval in 1..100i32) {
        let _ = overlapping_generations_streaming_simplification(seed, pdeath, simplify_interval);
    }
}
