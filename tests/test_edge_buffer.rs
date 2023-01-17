#![cfg(feature = "edgebuffer")]

use std::ops::Deref;

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
    fn start_recording(&mut self, _parents: &[NodeId], _child: &[NodeId]) {}
    fn end_recording(&mut self) {}
}

struct TableCollectionWithBuffer {
    tables: TableCollection,
    buffer: EdgeBuffer,
}

impl TableCollectionWithBuffer {
    fn new() -> Self {
        Self {
            tables: TableCollection::new(1.0).unwrap(),
            buffer: EdgeBuffer::default(),
        }
    }
}

impl Recording for TableCollectionWithBuffer {
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

impl From<TableCollectionWithBuffer> for TreeSequence {
    fn from(value: TableCollectionWithBuffer) -> Self {
        let mut value = value;
        value.tables.build_index().unwrap();
        value.tables.tree_sequence(0.into()).unwrap()
    }
}

struct StandardTableCollection(TableCollection);

impl StandardTableCollection {
    fn new() -> Self {
        Self(TableCollection::new(1.0).unwrap())
    }
}

struct TableCollectionWithBufferForStreaming {
    tables: TableCollection,
    buffer: EdgeBuffer,
    node_map: Vec<NodeId>,
}

impl TableCollectionWithBufferForStreaming {
    fn new() -> Self {
        Self {
            tables: TableCollection::new(1.0).unwrap(),
            buffer: EdgeBuffer::default(),
            node_map: vec![],
        }
    }
}

impl Recording for TableCollectionWithBufferForStreaming {
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
        self.buffer.buffer_birth(parent, child, left, right)
    }

    fn simplify(&mut self, samples: &mut [NodeId]) -> Result<(), TskitError> {
        self.node_map.resize(
            self.tables.nodes().num_rows().as_usize(),
            tskit::NodeId::NULL,
        );
        tskit::simplfify_from_buffer(
            &samples,
            tskit::SimplificationOptions::default(),
            &mut self.tables,
            &mut self.buffer,
            Some(&mut self.node_map),
        )
        .unwrap();
        for o in samples.iter_mut() {
            assert!(o.as_usize() < self.node_map.len());
            *o = self.node_map[usize::try_from(*o).unwrap()];
            assert!(!o.is_null());
        }
        self.buffer
            .post_simplification(&samples, &mut self.tables)
            .unwrap();
        Ok(())
    }
}

impl From<TableCollectionWithBufferForStreaming> for TreeSequence {
    fn from(value: TableCollectionWithBufferForStreaming) -> Self {
        let mut value = value;
        value.tables.build_index().unwrap();
        value.tables.tree_sequence(0.into()).unwrap()
    }
}

impl Recording for StandardTableCollection {
    fn add_node(&mut self, flags: u32, time: f64) -> Result<NodeId, TskitError> {
        self.0.add_node(flags, time, -1, -1)
    }
    fn add_edge(
        &mut self,
        left: f64,
        right: f64,
        parent: NodeId,
        child: NodeId,
    ) -> Result<(), TskitError> {
        match self.0.add_edge(left, right, parent, child) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }
    fn simplify(&mut self, samples: &mut [NodeId]) -> Result<(), TskitError> {
        self.0.full_sort(0).unwrap();
        match self.0.simplify(samples, 0, true) {
            Ok(Some(idmap)) => {
                for s in samples {
                    *s = idmap[s.as_usize()];
                }
                Ok(())
            }
            Ok(None) => Ok(()),
            Err(e) => Err(e),
        }
    }
}

impl From<StandardTableCollection> for TreeSequence {
    fn from(value: StandardTableCollection) -> Self {
        let mut value = value;
        value.0.build_index().unwrap();
        value.0.tree_sequence(0.into()).unwrap()
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
        use streaming_iterator::StreamingIterator;
        let standard = StandardTableCollection::new();
        let standard_treeseq = overlapping_generations(seed, pdeath, simplify_interval, standard);
        let with_buffer = TableCollectionWithBuffer::new();
        let standard_with_buffer = overlapping_generations(seed, pdeath, simplify_interval, with_buffer);
        let with_buffer_streaming = TableCollectionWithBufferForStreaming::new();
        let standard_with_buffer_streaming = overlapping_generations(seed, pdeath, simplify_interval, with_buffer_streaming);

        assert_eq!(standard_treeseq.num_trees(), standard_with_buffer.num_trees());
        assert_eq!(standard_treeseq.num_trees(), standard_with_buffer_streaming.num_trees());

        // cannot do KC distance b/c trees not fully coalesced.
        let mut trees_standard = standard_treeseq.tree_iterator(0).unwrap();
        let mut trees_with_buffer = standard_with_buffer.tree_iterator(0).unwrap();
        let mut trees_with_buffer_streaming = standard_with_buffer_streaming.tree_iterator(0).unwrap();

        while let Some(tree) = trees_standard.next() {
            let tree_with_buffer = trees_with_buffer.next().unwrap();
            assert_eq!(tree.interval(), tree_with_buffer.interval());
            //assert_eq!(tree.total_branch_length(true).unwrap(), tree_with_buffer.total_branch_length(true).unwrap());
            let tree_with_buffer_streaming = trees_with_buffer_streaming.next().unwrap();
            assert_eq!(tree.interval(), tree_with_buffer_streaming.interval());
            //assert_eq!(tree.total_branch_length(true).unwrap(), tree_with_buffer_streaming.total_branch_length(true).unwrap());
        }
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
