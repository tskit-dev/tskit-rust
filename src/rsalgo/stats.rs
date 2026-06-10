use crate::MutationId;
use crate::MutationRef;
use crate::NodeId;
use crate::Position;
use crate::SiteId;
use crate::TableColumn;
use crate::TreeSequence;

pub trait SingleSiteStatistic {
    fn update(&mut self, num_descendants: i64);
}

#[derive(Debug)]
pub enum StatsError {
    MissingAncestralState(SiteId),
    MissingDerivedState(MutationId),
    InvalidNode(NodeId),
}

impl std::fmt::Display for StatsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "stats error")
    }
}

impl std::error::Error for StatsError {}

// Poperties of the current tree with respect to
// a given sample set.
// Multi-sample-set tasks will need a Vec of these.
struct TreeData {
    // NOTE: we manually handle all tskit conventions:
    // * i32 for ids
    // * -1 for "NULL" value
    parent: Vec<i32>,
    num_sample_descendants: Vec<i64>,
    num_mutated_sample_descendants: Vec<i64>,
}

impl TreeData {
    fn new(ts: &TreeSequence) -> Self {
        Self {
            parent: vec![-1; ts.nodes().num_rows().as_usize()],
            num_sample_descendants: vec![0; ts.nodes().num_rows().as_usize()],
            num_mutated_sample_descendants: vec![0; ts.mutations().num_rows().as_usize()],
        }
    }

    fn update_ancestors(&mut self, p: i32, delta: i64) {
        let mut p = p;
        while p != -1 {
            self.num_sample_descendants[p as usize] += delta;
            debug_assert!(self.num_sample_descendants[p as usize] >= 0);
            p = self.parent[p as usize];
        }
    }

    fn process_output_edge(&mut self, parent: usize, child: usize) {
        debug_assert!(
            self.num_sample_descendants[child] <= self.num_sample_descendants[parent],
            "{parent} ({}) -> {child} ({})",
            self.num_sample_descendants[parent],
            self.num_sample_descendants[child],
        );
        if self.num_sample_descendants[child] > 0 {
            let delta = -self.num_sample_descendants[child];
            self.update_ancestors(self.parent[child], delta);
        }
        self.parent[child] = -1;
    }

    fn process_input_edge(&mut self, parent: usize, child: usize) {
        debug_assert!(self.num_sample_descendants[parent] >= 0);
        debug_assert!(self.num_sample_descendants[child] >= 0);
        self.parent[child] = parent as i32;
        if self.num_sample_descendants[child] > 0 {
            let delta = self.num_sample_descendants[child];
            self.update_ancestors(self.parent[child], delta);
        }
    }

    // This is intended as a truly private fn
    fn get_num_sample_descendants(&mut self, mutation: &MutationRef<'_>) -> i64 {
        let rv: i64 = self.num_sample_descendants[mutation.node().as_usize()]
            - self.num_mutated_sample_descendants[mutation.id().as_usize()];
        // this is a HARD error representing a serious bug.
        assert!(rv >= 0);
        rv
    }

    #[must_use]
    fn process_mutation<M>(&mut self, mutation: &MutationRef<'_>, mutation_parent: &M) -> i64
    where
        M: TableColumn<MutationId, MutationId>,
    {
        let nd = self.get_num_sample_descendants(mutation);
        if nd > 0 {
            // Propagate number of nodes inheriting this mutation up the tree
            let delta = self.num_sample_descendants[mutation.node().as_usize()]
                - self.num_mutated_sample_descendants[mutation.id().as_usize()];
            assert!(!delta.is_negative());
            let mut current_mut_parent = mutation_parent[mutation.id()];
            while !current_mut_parent.is_null() {
                self.num_mutated_sample_descendants[current_mut_parent.as_usize()] += delta;
                current_mut_parent = mutation_parent[current_mut_parent];
            }
        }
        nd
    }
}

trait SampleSets<'s> {
    fn process_input_edge(&mut self, parent: usize, child: usize);
    fn process_output_edge(&mut self, parent: usize, child: usize);
    fn initialize_site<'ts, 'a>(
        &'a mut self,
        ts: &'ts TreeSequence,
        site: SiteId,
    ) -> Result<(), StatsError>
    where
        'ts: 's,
        's: 'a;
    fn process_mutation<'ts, 'a, M>(
        &'a mut self,
        ts: &'ts TreeSequence,
        mutation_parent: &'a M,
        mutation: MutationRef<'a>,
    ) -> Result<(), StatsError>
    where
        'ts: 's,
        's: 'a,
        M: TableColumn<MutationId, MutationId>;
    fn update_allele_counts(&mut self) -> Result<(), StatsError>;
}

struct SingleSampleSet<'ts> {
    tree_data: TreeData,
    num_sampled_genomes: i64,
    alleles_at_site: Vec<&'ts [u8]>,
    allele_counts: Vec<i64>,
}

impl<'s> SampleSets<'s> for SingleSampleSet<'s> {
    fn process_input_edge(&mut self, parent: usize, child: usize) {
        self.tree_data.process_input_edge(parent, child);
    }
    fn process_output_edge(&mut self, parent: usize, child: usize) {
        self.tree_data.process_output_edge(parent, child);
    }

    fn process_mutation<'ts, 'a, M>(
        &'a mut self,
        ts: &'ts TreeSequence,
        mutation_parent: &'a M,
        mutation: MutationRef<'a>,
    ) -> Result<(), StatsError>
    where
        'ts: 's,
        's: 'a,
        M: TableColumn<MutationId, MutationId>,
    {
        let num_samples_inheriting_derived_state =
            self.tree_data.process_mutation(&mutation, mutation_parent);
        if num_samples_inheriting_derived_state > 0
            && num_samples_inheriting_derived_state < self.num_sampled_genomes
        {
            let derived_state = *ts
                .mutations()
                .derived_state(mutation.id())
                .as_ref()
                .ok_or(StatsError::MissingDerivedState(mutation.id()))?;
            match self
                .alleles_at_site
                .iter()
                .position(|&x| x == derived_state)
            {
                Some(index) => {
                    if index > 0 {
                        self.allele_counts[index] += num_samples_inheriting_derived_state
                    }
                }
                None => {
                    self.alleles_at_site.push(derived_state);
                    self.allele_counts
                        .push(num_samples_inheriting_derived_state);
                }
            }
        }
        Ok(())
    }

    fn update_allele_counts(&mut self) -> Result<(), StatsError> {
        self.allele_counts[0] =
        // TODO: we should simply sum the desired quantity as we go along,
        // eliminating the need for an iteration here.
            (self.num_sampled_genomes) - self.allele_counts.iter().skip(1).sum::<i64>();
        assert!(self.allele_counts[0] >= 0);
        if self
            .allele_counts
            .iter()
            .filter(|&&i| i > 0 && i < self.num_sampled_genomes)
            .count()
            > 1
        {
            self.counts
                .add_site_from_counts(&self.allele_counts, self.num_sampled_genomes as i32)?;
        }
        Ok(())
    }

    fn initialize_site<'ts, 'a>(
        &'a mut self,
        ts: &'ts TreeSequence,
        site: SiteId,
    ) -> Result<(), StatsError>
    where
        'ts: 's,
        's: 'a,
    {
        setup_alleles_at_site(ts, site, &mut self.alleles_at_site)?;
        self.allele_counts.resize(1, 0);
        Ok(())
    }
}

fn setup_alleles_at_site<'ts, 'a>(
    ts: &'ts TreeSequence,
    site: SiteId,
    alleles_at_site: &mut Vec<&'a [u8]>,
) -> Result<(), StatsError>
where
    'ts: 'a,
{
    alleles_at_site.clear();
    // NOTE: trying to store the derived state
    // from the current_site as a slice runs
    // into lifetime issues because current_site
    // goes away. So what we do instead is get a slice
    // for the same row whose lifetime depends on
    // the tree sequence!
    alleles_at_site.push(
        *ts.sites()
            .ancestral_state(site)
            .as_ref()
            .ok_or(StatsError::MissingAncestralState(site))?,
    );
    Ok(())
}

fn setup_samples_from_node_ids<I>(
    num_nodes: usize,
    iter: I,
    td: &mut TreeData,
) -> Result<i32, StatsError>
where
    I: Iterator<Item = NodeId>,
{
    let mut num_sampled_genomes = 0;
    for node_id in iter {
        // Should be an Err condition!
        if node_id == NodeId::NULL {
            return Err(StatsError::InvalidNode(node_id));
        }
        // Should be an Err condition!
        assert!(node_id.as_usize() < num_nodes);
        if let Some(value) = td.num_sample_descendants.get_mut(node_id.as_usize()) {
            *value += 1;
        } else {
            return Err(StatsError::InvalidNode(node_id));
        }
        num_sampled_genomes += 1;
    }
    Ok(num_sampled_genomes)
}

pub fn single_site_statistic<N: Iterator<Item = NodeId>, S: SingleSiteStatistic>(
    samples: N,
    statistic: S,
    ts: &TreeSequence,
) {
    let mutation_parent = ts.tables().mutations().parent_column();
    let num_edges = ts.edges().num_rows().as_usize();
}
