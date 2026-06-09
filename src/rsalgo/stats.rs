use crate::NodeId;

pub trait SingleSiteStatistic {
    fn update(&mut self, num_descendants: i64);
}

pub fn single_site_statistic<N: Iterator<Item = NodeId>, S: SingleSiteStatistic>(
    samples: N,
    statistic: S,
) {
}
