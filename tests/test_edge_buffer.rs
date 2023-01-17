#![cfg(feature = "edgebuffer")]

use proptest::prelude::*;
use rand::distributions::Distribution;
use rand::SeedableRng;

use tskit::EdgeBuffer;
use tskit::EdgeId;
use tskit::NodeId;
use tskit::TableCollection;
use tskit::TreeSequence;
use tskit::TskitError;

trait Recording {
    fn add_node(&mut self, flags: u32, time: f64) -> Result<NodeId, TskitError>;
    fn add_edge(
        &mut self,
        left: f64,
        right: f64,
        parent: NodeId,
        child: NodeId,
    ) -> Result<(), TskitError>;

    fn simplify(&mut self, samples: &mut [NodeId]) -> Result<(), TskitError>;
    fn start_recording(&mut self, parents: &[NodeId], child: &[NodeId]) {}
    fn end_recording(&mut self) {}
}

struct StandardTableCollectionWithBuffer {
    tables: TableCollection,
    buffer: EdgeBuffer,
}

impl StandardTableCollectionWithBuffer {
    fn new() -> Self {
        Self {
            tables: TableCollection::new(1.0).unwrap(),
            buffer: EdgeBuffer::default(),
        }
    }
}

impl Recording for StandardTableCollectionWithBuffer {
    fn add_node(&mut self, flags: u32, time: f64) -> Result<NodeId, TskitError> {
        self.tables.add_node(flags, time, -1, -1)
    }

    fn add_edge(
        &mut self,
        left: f64,
        right: f64,
        parent: NodeId,
        child: NodeId,
    ) -> Result<(), TskitError> {
        self.buffer.record_birth(parent, child, left, right)
    }

    fn start_recording(&mut self, parents: &[NodeId], children: &[NodeId]) {
        self.buffer.setup_births(parents, children).unwrap()
    }

    fn end_recording(&mut self) {
        self.buffer.finalize_births()
    }

    fn simplify(&mut self, samples: &mut [NodeId]) -> Result<(), TskitError> {
        self.buffer.pre_simplification(&mut self.tables).unwrap();
        match self.tables.simplify(samples, 0, true) {
            Ok(Some(idmap)) => {
                for s in samples.iter_mut() {
                    *s = idmap[s.as_usize()];
                }
                self.buffer
                    .post_simplification(samples, &mut self.tables)
                    .unwrap();
                Ok(())
            }
            Ok(None) => panic!(),
            Err(e) => Err(e),
        }
    }
}

impl From<StandardTableCollectionWithBuffer> for TreeSequence {
    fn from(value: StandardTableCollectionWithBuffer) -> Self {
        let mut value = value;
        value.tables.build_index().unwrap();
        value.tables.tree_sequence(0.into()).unwrap()
    }
}

fn overlapping_generations<T>(seed: u64, pdeath: f64, simplify: i32, recorder: T) -> TreeSequence
where
    T: Into<TreeSequence> + Recording,
{
    let mut rng = rand::rngs::StdRng::seed_from_u64(seed);

    let popsize = 10;

    let mut parents = vec![];

    let mut recorder = recorder;

    for _ in 0..popsize {
        let node = recorder.add_node(0, 10.0).unwrap();
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
            let child = recorder.add_node(0, birth_time as f64).unwrap();
            births.push(child);
            recorder.start_recording(&[parent], &[child]);
            recorder.add_edge(0., 1., parent, child).unwrap();
            recorder.end_recording();
        }

        for (r, b) in replacements.iter().zip(births.iter()) {
            assert!(*r < parents.len());
            parents[*r] = *b;
        }
        if birth_time % simplify == 0 {
            recorder.simplify(&mut parents).unwrap();
        }
    }
    recorder.into()
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
        assert_eq!(replacements.len(), births.len());

        for (r, b) in replacements.iter().zip(births.iter()) {
            assert!(*r < parents.len());
            parents[*r] = *b;
        }
        if birth_time % simplify == 0 {
            node_map.resize(tables.nodes().num_rows().as_usize(), tskit::NodeId::NULL);
            tskit::simplfify_from_buffer(
                &parents,
                tskit::SimplificationOptions::default(),
                &mut tables,
                &mut buffer,
                Some(&mut node_map),
            )
            .unwrap();
            for o in parents.iter_mut() {
                assert!(o.as_usize() < node_map.len());
                *o = node_map[usize::try_from(*o).unwrap()];
                assert!(!o.is_null());
            }
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
        let with_buffer = StandardTableCollectionWithBuffer::new();
        let _ = overlapping_generations(seed, pdeath, simplify_interval, with_buffer);
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
