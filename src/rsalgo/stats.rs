use crate::MutationId;
use crate::NodeId;
use crate::Position;
use crate::TreeSequence;

pub trait SingleSiteStatistic {
    fn update(&mut self, num_descendants: i64);
}

pub struct SingleSiteStatisticState<'ts, S: SingleSiteStatistic> {
    edges_parent: &'ts [NodeId],
    edges_child: &'ts [NodeId],
    edges_left: &'ts [Position],
    edges_right: &'ts [Position],
    mutation_parent: &'ts [MutationId],
    statistic: S,
}

impl<'ts, S: SingleSiteStatistic> SingleSiteStatisticState<'ts, S> {
    fn new(treeseq: &'ts TreeSequence, statistic: S) -> Self {
        todo!()
    }
}

impl<'ts, S: SingleSiteStatistic> super::incremental_algorithm::IncrementalAlgorithm
    for SingleSiteStatisticState<'ts, S>
{
    fn process_output_edge(&mut self, parent: NodeId, child: NodeId) {
        todo!()
    }
    fn process_input_edge(&mut self, parent: NodeId, child: NodeId) {
        todo!()
    }
    fn process_interval(&mut self, left: crate::Position, right: crate::Position) {
        todo!()
    }
}

pub fn single_site_statistic<N: Iterator<Item = NodeId>, S: SingleSiteStatistic>(
    samples: N,
    statistic: S,
    ts: &TreeSequence,
) {
}
