#[cfg(test)]
use crate::*;

#[cfg(test)]
#[derive(serde::Serialize, serde::Deserialize, Debug, Eq, PartialEq, Copy, Clone)]
pub struct GenericMetadata {
    pub data: i64,
}

#[cfg(test)]
impl Default for GenericMetadata {
    fn default() -> Self {
        Self { data: 42 }
    }
}

#[cfg(test)]
impl crate::metadata::MetadataRoundtrip for GenericMetadata {
    fn encode(&self) -> Result<Vec<u8>, crate::metadata::MetadataError> {
        handle_metadata_return!(bincode::serialize(&self))
    }

    fn decode(md: &[u8]) -> Result<Self, crate::metadata::MetadataError> {
        handle_metadata_return!(bincode::deserialize(md))
    }
}

#[cfg(test)]
impl crate::metadata::MutationMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::SiteMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::EdgeMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::NodeMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::IndividualMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::PopulationMetadata for GenericMetadata {}

#[cfg(test)]
impl crate::metadata::MigrationMetadata for GenericMetadata {}

#[cfg(test)]
pub mod bad_metadata {
    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct F {
        pub x: i32,
        pub y: u32,
    }

    #[derive(serde::Serialize, serde::Deserialize, Debug)]
    pub struct Ff {
        pub x: i32,
        pub y: u64,
    }

    impl crate::metadata::MetadataRoundtrip for F {
        fn encode(&self) -> Result<Vec<u8>, crate::metadata::MetadataError> {
            handle_metadata_return!(bincode::serialize(&self))
        }

        fn decode(md: &[u8]) -> Result<Self, crate::metadata::MetadataError> {
            handle_metadata_return!(bincode::deserialize(md))
        }
    }

    impl crate::metadata::MetadataRoundtrip for Ff {
        fn encode(&self) -> Result<Vec<u8>, crate::metadata::MetadataError> {
            handle_metadata_return!(bincode::serialize(&self))
        }

        fn decode(md: &[u8]) -> Result<Self, crate::metadata::MetadataError> {
            handle_metadata_return!(bincode::deserialize(md))
        }
    }

    impl crate::metadata::MutationMetadata for F {}
    impl crate::metadata::MutationMetadata for Ff {}
}

/// mimic the c simulate function in tskit c api document
/// https://tskit.dev/tskit/docs/stable/c-api.html#basic-forwards-simulator
#[cfg(test)]
pub mod simulation {
    use core::panic;

    use crate::{
        metadata::{MetadataError, MetadataRoundtrip, PopulationMetadata},
        EdgeId, IndividualId, MutationId, NodeFlags, NodeId, PopulationId, Position,
        SimplificationOptions, SiteId, TableCollection, TableSortOptions, TreeSequence,
        TreeSequenceFlags, TskitError,
    };
    use rand::{rngs::StdRng, Rng, SeedableRng};

    struct MyMeta {
        inner: String,
    }
    impl From<String> for MyMeta {
        fn from(value: String) -> Self {
            MyMeta { inner: value }
        }
    }
    impl<'a> From<&'a str> for MyMeta {
        fn from(value: &'a str) -> Self {
            MyMeta {
                inner: value.to_owned(),
            }
        }
    }

    // helper structs, impls and functions

    impl MetadataRoundtrip for MyMeta {
        fn encode(&self) -> Result<Vec<u8>, MetadataError> {
            Ok(self.inner.as_bytes().to_owned())
        }
        fn decode(md: &[u8]) -> Result<Self, MetadataError>
        where
            Self: Sized,
        {
            Ok(MyMeta {
                inner: String::from_utf8(md.to_owned()).unwrap(),
            })
        }
    }

    impl PopulationMetadata for MyMeta {}

    fn add_pop(tables: &mut TableCollection, name: &str) -> PopulationId {
        tables
            .add_population_with_metadata(&MyMeta::from(name))
            .unwrap()
    }

    fn add_ind(
        tables: &mut TableCollection,
        parent1: (NodeId, NodeId),
        parent2: (NodeId, NodeId),
    ) -> IndividualId {
        let parent1_ind = tables.nodes().individual(parent1.0).unwrap();
        let parent2_ind = tables.nodes().individual(parent2.0).unwrap();
        let flags = 0u32;
        let loc_null = None;
        tables
            .add_individual(flags, loc_null, [parent1_ind, parent2_ind])
            .unwrap()
    }

    fn find_parent(
        rng: &mut StdRng,
        parents: &[(NodeId, NodeId)],
        child_pop: PopulationId,
    ) -> ((NodeId, NodeId), PopulationId) {
        assert_eq!(parents.len() % 2, 0);
        let (pop_anc, pop_1, pop_2) = (0, 1, 2);
        let child_pop: i32 = child_pop.into();

        let pop_size = parents.len();
        let mut parent_pop = child_pop;

        let is_migrant = (child_pop != pop_anc) && rng.gen_bool(0.01);
        if is_migrant {
            parent_pop = if child_pop == pop_1 { pop_2 } else { pop_1 };
        };
        let parent = match parent_pop {
            // pop_anc
            0 => parents[rng.gen_range(0..pop_size)],
            // pop_1
            1 => parents[rng.gen_range(0..(pop_size / 2))],
            // pop_2
            2 => parents[rng.gen_range((pop_size / 2)..pop_size)],
            _ => panic!("wrong population id encountered"),
        };
        (parent, parent_pop.into())
    }

    fn find_breakpoint(rng: &mut StdRng, seqlen: Position) -> Position {
        // avoid breaking as edges
        let seqlen = f64::from(seqlen).floor() as usize;
        let sel = rng.gen_range(1..seqlen) as f64;
        Position::from(sel)
    }

    fn add_node(
        tables: &mut TableCollection,
        is_sample: bool,
        time: usize,
        pop: PopulationId,
        ind: IndividualId,
    ) -> NodeId {
        tables
            .add_node(
                if is_sample {
                    NodeFlags::new_sample()
                } else {
                    NodeFlags::default()
                },
                time as f64,
                pop,
                ind,
            )
            .unwrap()
    }

    fn add_edge(
        tables: &mut TableCollection,
        start: impl Into<Position>,
        end: impl Into<Position>,
        parent_node: NodeId,
        child_node: NodeId,
    ) -> EdgeId {
        tables
            .add_edge(start, end, parent_node, child_node)
            .unwrap()
    }

    fn find_overlaps<P>(start: P, end: P, intervals: &Vec<(P, P)>, out: &mut Vec<(P, P)>)
    where
        P: Into<Position> + Copy + PartialOrd,
    {
        // assert intervals is sorted
        assert!(intervals.iter().all(|(a, b)| *a <= *b));
        assert!(intervals
            .iter()
            .zip(intervals.iter().skip(1))
            .all(|(p1, p2)| p1.1 <= p2.0));
        // clear out
        out.clear();

        for (m, n) in intervals {
            // no overlap
            if (*n <= start) || (end <= *m) {
                continue;
            }
            let new_start = if *m < start { start } else { *m };
            let new_end = if *n < end { *n } else { end };
            out.push((new_start, new_end));
        }
    }

    fn find_mutation_pos<P>(rng: &mut StdRng, s: P, e: P) -> usize
    where
        P: Into<Position>,
    {
        let s = f64::from(Into::<Position>::into(s)).ceil() as usize;
        let e = f64::from(Into::<Position>::into(e)).floor() as usize;
        rng.gen_range(s..e)
    }

    fn calc_derived_state(site_last_mutation_order: &[usize], mut_pos: usize) -> [u8; 1] {
        [b'a'
            + match site_last_mutation_order[mut_pos] + 1 {
                x if x > 45 => 45u8,
                x => x as u8,
            }]
    }

    /// simulate diplid individual with migration between two subpopulations
    ///
    /// Both full_trees and trucated_trees will be generated
    pub fn simulate_two_treesequences<P>(
        seqlen: P,
        pop_size: usize,
        start_time: usize,
        split_time: usize,
        intervals: &[(P, P)],
        seed: u64,
    ) -> Result<(TreeSequence, TreeSequence), TskitError>
    where
        P: Into<Position> + Copy + PartialOrd,
    {
        let rng = &mut StdRng::seed_from_u64(seed);
        let intervals: Vec<(Position, Position)> = intervals
            .iter()
            .map(|(a, b)| ((*a).into(), (*b).into()))
            .collect();
        assert!(split_time < start_time);
        assert_eq!(pop_size % 2, 0);
        // tables without truncation
        let mut tables = TableCollection::new(seqlen).unwrap();
        // expected tables after truncation
        // it is built following `tables` except for positions for edge table
        let mut tr_tbls = TableCollection::new(seqlen).unwrap();

        let mut buffer = Vec::new();

        // add pop
        let pop_anc = add_pop(&mut tables, "ancestor");
        let pop_1 = add_pop(&mut tables, "pop1");
        let pop_2 = add_pop(&mut tables, "pop2");

        add_pop(&mut tr_tbls, "ancestral");
        add_pop(&mut tr_tbls, "pop1");
        add_pop(&mut tr_tbls, "pop2");

        // state variables for site/mutation tables
        let num_sites = f64::from(seqlen.into()) as usize;
        let mut site_last_mutation_order = vec![0usize; num_sites];

        let mut site_last_mutation_tables = vec![MutationId::NULL; num_sites];
        let mut site_last_mutation_tr_tbls = vec![MutationId::NULL; num_sites];

        let mut site_id_map_tables = vec![SiteId::NULL; num_sites];
        let mut site_id_map_tr_tbls = vec![SiteId::NULL; num_sites];

        // base population
        let mut parents = Vec::<(NodeId, NodeId)>::with_capacity(pop_size);
        for _ in 0..pop_size {
            const FLAGS: u32 = 0;
            let loc_null = None;
            let parent_ind = tables.add_individual(FLAGS, loc_null, None).unwrap();
            tr_tbls.add_individual(FLAGS, loc_null, None).unwrap();

            let parent_id = (
                add_node(&mut tables, false, start_time, pop_anc, parent_ind),
                add_node(&mut tables, false, start_time, pop_anc, parent_ind),
            );
            parents.push(parent_id);
            //
            add_node(&mut tr_tbls, false, start_time, pop_anc, parent_ind);
            add_node(&mut tr_tbls, false, start_time, pop_anc, parent_ind);
        }

        // offspring population
        let mut children = Vec::<(NodeId, NodeId)>::with_capacity(pop_size);

        for t in (0..start_time).rev() {
            for i in 0..pop_size {
                // select breakpoints
                let breakpoint1 = find_breakpoint(rng, seqlen.into());
                let breakpoint2 = find_breakpoint(rng, seqlen.into());

                // find child pop
                let mut child_pop = pop_anc;
                if t > split_time {
                    child_pop = if i < pop_size / 2 { pop_1 } else { pop_2 }
                }

                // find parents
                let (parent1, _parent1_pop) = find_parent(rng, &parents, child_pop);
                let (parent2, _parent2_pop) = find_parent(rng, &parents, child_pop);

                // add individual
                let child_ind = add_ind(&mut tables, parent1, parent2);
                add_ind(&mut tr_tbls, parent1, parent2);

                // add nodes
                let is_sample = t == 0;
                let child_id = (
                    add_node(&mut tables, is_sample, t, child_pop, child_ind),
                    add_node(&mut tables, is_sample, t, child_pop, child_ind),
                );

                add_node(&mut tr_tbls, is_sample, t, child_pop, child_ind);
                add_node(&mut tr_tbls, is_sample, t, child_pop, child_ind);

                // add edges, sites & mutations to both tables and tr_tabls
                let mu = 0.01f64;
                for (s, e, p, c) in [
                    (0.0.into(), breakpoint1, parent1.0, child_id.0),
                    (breakpoint1, seqlen.into(), parent1.1, child_id.0),
                    (0.0.into(), breakpoint2, parent2.0, child_id.1),
                    (breakpoint2, seqlen.into(), parent2.1, child_id.1),
                ] {
                    add_edge(&mut tables, s, e, p, c);

                    let mut_pos = find_mutation_pos(rng, s, e);
                    let mut mut_prob = f64::from(e - s) * mu;
                    if mut_prob > 1.0 {
                        mut_prob = 1.0;
                    }
                    let to_add_mut: bool = rng.gen_bool(mut_prob);
                    let derived_state = &calc_derived_state(&site_last_mutation_order, mut_pos);
                    let t = t as f64;

                    if to_add_mut {
                        // add site
                        let site_not_exist = site_id_map_tables[mut_pos] == SiteId::NULL;
                        if site_not_exist {
                            site_id_map_tables[mut_pos] =
                                tables.add_site(mut_pos as f64, Some(&[b'a'])).unwrap();
                        }
                        // add mutation
                        let parent_mut = site_last_mutation_tables[mut_pos];
                        let site = site_id_map_tables[mut_pos];
                        let new_mutation = tables
                            .add_mutation(site, c, parent_mut, t, Some(derived_state))
                            .unwrap();

                        site_last_mutation_tables[mut_pos] = new_mutation;
                        site_last_mutation_order[mut_pos] += 1;
                    }

                    find_overlaps(s, e, &intervals, &mut buffer);
                    for (s_, e_) in buffer.iter() {
                        add_edge(&mut tr_tbls, *s_, *e_, p, c);
                        let mut_pos_f = mut_pos as f64;

                        if to_add_mut && (*s_ <= mut_pos_f) && (*e_ > mut_pos_f) {
                            // add site
                            let site_not_exist = site_id_map_tr_tbls[mut_pos] == SiteId::NULL;
                            if site_not_exist {
                                site_id_map_tr_tbls[mut_pos] =
                                    tr_tbls.add_site(mut_pos as f64, Some(&[b'a'])).unwrap();
                            }
                            // add mutation
                            let parent_mut = site_last_mutation_tr_tbls[mut_pos];
                            let site = site_id_map_tr_tbls[mut_pos];
                            let new_mutation = tr_tbls
                                .add_mutation(site, c, parent_mut, t, Some(derived_state))
                                .unwrap();
                            site_last_mutation_tr_tbls[mut_pos] = new_mutation;
                        }
                    }
                }

                // add edges for tr_tbls
                children.push(child_id);
            }
            // NOTE: avoid simplifcation so that both tables and tr_tables share the same ids

            // set children as parents and clear children
            std::mem::swap(&mut children, &mut parents);
            children.clear();
        }

        let sort_opts = TableSortOptions::all();
        tables.full_sort(sort_opts).unwrap();
        tr_tbls.full_sort(sort_opts).unwrap();

        // simplify
        let mut samples = Vec::<NodeId>::with_capacity(pop_size * 2);
        parents
            .iter()
            .for_each(|p| samples.extend([p.0, p.1].iter()));

        let simplify_opts = SimplificationOptions::default();
        tables.simplify(&samples, simplify_opts, false).unwrap();
        tr_tbls.simplify(&samples, simplify_opts, false).unwrap();

        // build indices
        tables.build_index().unwrap();
        tr_tbls.build_index().unwrap();

        // to tree sequences
        let treeseq_opts = TreeSequenceFlags::default();
        let full_trees = TreeSequence::new(tables, treeseq_opts).unwrap();
        let truncated_trees = TreeSequence::new(tr_tbls, treeseq_opts).unwrap();

        Ok((full_trees, truncated_trees))
    }

    pub fn generate_simple_treesequence(add_migration_records: bool) -> TreeSequence {
        let snode = NodeFlags::new_sample();
        let anode = NodeFlags::default();
        let pop = PopulationId::NULL;
        let ind = IndividualId::NULL;
        let seqlen = 100.0;
        let (t0, t10) = (0.0, 10.0);
        let (left, right) = (0.0, 100.0);

        let sim_opts = SimplificationOptions::default();
        let mut tables = TableCollection::new(seqlen).unwrap();
        let child1 = tables.add_node(snode, t0, pop, ind).unwrap();
        let child2 = tables.add_node(snode, t0, pop, ind).unwrap();
        let parent = tables.add_node(anode, t10, pop, ind).unwrap();
        tables.add_edge(left, right, parent, child1).unwrap();
        tables.add_edge(left, right, parent, child2).unwrap();

        tables.full_sort(TableSortOptions::all()).unwrap();
        let id_map = tables
            .simplify(&[child1, child2], sim_opts, true)
            .unwrap()
            .unwrap()
            .to_owned();

        // add migration records after simplification to avoid errors when
        // simplifying a treesequence that contains a nonempty migration table
        if add_migration_records {
            let pop_anc = tables.add_population().unwrap();
            let pop_1 = tables.add_population().unwrap();
            let pop_2 = tables.add_population().unwrap();
            // get new ids after simplifcation
            let child1 = id_map[child1.to_usize().unwrap()];
            let child2 = id_map[child2.to_usize().unwrap()];
            tables
                .add_migration((left, right), child1, (pop_anc, pop_1), t0 + 1.0)
                .unwrap();
            tables
                .add_migration((left, right), child2, (pop_anc, pop_2), t0 + 5.0)
                .unwrap();
        }

        tables.build_index().unwrap();

        let flags = TreeSequenceFlags::default();
        TreeSequence::new(tables, flags).unwrap()
    }
}

#[cfg(test)]
mod keep_intervals {
    use crate::*;

    use super::simulation::{generate_simple_treesequence, simulate_two_treesequences};

    #[test]
    fn test_keep_intervals_invalid_input() {
        let intervals_lst = vec![
            vec![(20.0, 10.0)],               // out of order
            vec![(10.0, 20.0), (19.0, 30.0)], // overlapping intervals
        ];
        for intervals in intervals_lst {
            let add_migration_table = false;
            let trees = generate_simple_treesequence(add_migration_table);
            let res = trees.keep_intervals(intervals.into_iter());
            assert!(res.is_err());
        }
    }

    #[test]
    fn test_keep_intervals_nonempty_migration_table() {
        let intervals = [(10.0, 20.0)];

        let add_migration_table = true;
        let trees = generate_simple_treesequence(add_migration_table);
        let res = trees.keep_intervals(intervals.iter().copied());
        assert!(res.is_ok());

        let add_migration_table = false;
        let trees = generate_simple_treesequence(add_migration_table);
        let res = trees.keep_intervals(intervals.iter().copied());
        assert!(res.is_ok());
    }

    #[test]
    fn test_keep_intervals() {
        let seqlen = 1000.0;
        let intervals_lst = vec![
            vec![(seqlen + 1.0, seqlen + 2.0)], // out of range: > seqlen
            vec![(10.0, 20.0), (700.0, 850.0)], // multiple intervals
            vec![(10.0, 20.0)],                 // single intervals
        ];
        let popsize = 50;
        let total_time = 300;
        let split_time = 20;

        for intervals in intervals_lst {
            for seed in [123, 3224] {
                let (full_trees, exepected) = simulate_two_treesequences(
                    seqlen, popsize, total_time, split_time, &intervals, seed,
                )
                .unwrap();

                if exepected.edges().num_rows() > 0 {
                    let mut truncated = full_trees
                        .keep_intervals(intervals.iter().copied())
                        .expect("error")
                        .expect("empty table");
                    let samples = truncated.samples_as_vector();
                    assert!(truncated.edges().num_rows() > 0);
                    truncated
                        .simplify(&samples, crate::SimplificationOptions::default(), false)
                        .expect("error simplifying");

                    // dump tables for comparision
                    let expected = exepected.dump_tables().unwrap();
                    let res = truncated.equals(&expected, TableEqualityOptions::all());
                    assert!(res);
                } else {
                    let trucated = full_trees
                        .keep_intervals(intervals.iter().copied())
                        .unwrap();
                    assert!(trucated.is_none());
                }
            }
        }
    }
}
