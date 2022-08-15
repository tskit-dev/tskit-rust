use clap::Parser;
/*
 * Forward simulation without selection:
 *
 * * Constant population size
 * * Poisson number of crossovers per parent, per mating.
 * * Overlapping generations supported.
 * * Simplifcation keeps input roots for later "recapitation".
 *
 * On the rust side:
 *
 * * Runtime errors are handled when they occur.
 *   We do this because stable rust does not have
 *   support for back traces.
 * * No "unsafe" tskit C code is called directly.
 * * By default, clap doesn't allow argument values to start
 *   with a hyphen ('-').  This is handy, as it automatically
 *   prevents negative population sizes, genome lengths,
 *   and numbers of crossovers, etc.., from being entered
 *   on the command line.
 */
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rand_distr::{Exp, Uniform};
use tskit::TableAccess;
use tskit::TskitTypeAccess;

#[derive(clap::Parser)]
struct SimParams {
    #[clap(
        short = 'N',
        long = "popsize",
        value_parser,
        default_value_t = 1000,
        help = "Diploid population size. Default = 1,000"
    )]
    pub popsize: u32,
    #[clap(
        short = 'n',
        long = "nsteps",
        value_parser,
        default_value_t = 1000,
        help = "Number of birth steps to simulate. For non-overlapping generations, this is the number of generations to simulate. Default = 1,000."
    )]
    pub nsteps: u32,
    #[clap(
        short = 'x',
        long = "xovers",
        value_parser,
        default_value_t = 0.0,
        help = "Mean number of crossovers per meiosis. The number of crossovers is Poisson-distributed with this value. Default = 0.0."
    )]
    pub xovers: f64,
    #[clap(
        short = 'P',
        long = "psurvival",
        value_parser,
        default_value_t = 0.0,
        help = "Survival probability. A value of 0.0 is the Wright-Fisher model of non-overlapping generations.  Values must b 0.0 <= p < 1.0.  Default = 0.0."
    )]
    pub psurvival: f64,
    #[clap(
        short = 'L',
        long = "genome_length",
        value_parser,
        default_value_t = 1e6,
        help = "Genome length (continuous units).  Default = 1e6."
    )]
    pub genome_length: f64,
    #[clap(
        short = 's',
        long = "simplification_interval",
        value_parser,
        default_value_t = 100,
        help = "Number of birth steps between simplifications. Default = 100."
    )]
    pub simplification_interval: u32,
    #[clap(
        short = 't',
        long = "treefile",
        value_parser,
        default_value = "treefile.trees",
        help = "Name of output file. The format is a tskit \"trees\" file. Default = \"treefile.trees\"."
    )]
    pub treefile: String,
    #[clap(
        short = 'S',
        long = "seed",
        value_parser,
        default_value_t = 0,
        help = "Random number seed. Default = 0."
    )]
    pub seed: u64,
}

impl Default for SimParams {
    fn default() -> Self {
        Self {
            popsize: 1000,
            nsteps: 1000,
            xovers: 0.,
            psurvival: 0.0,
            genome_length: 1e6,
            simplification_interval: 100,
            treefile: String::from("treefile.trees"),
            seed: 0,
        }
    }
}

#[derive(Debug, Clone)]
struct BadParameter {
    msg: String,
}

impl std::fmt::Display for BadParameter {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

impl SimParams {
    // NOTE: This function is incomplete.
    fn validate(&self) -> Result<(), BadParameter> {
        match self.psurvival.partial_cmp(&0.0) {
            Some(std::cmp::Ordering::Less) => {
                return Err(BadParameter {
                    msg: String::from("psurvival must be 0 <= p < 1.0"),
                });
            }
            Some(_) => (),
            None => (),
        }

        match self.psurvival.partial_cmp(&1.0) {
            Some(std::cmp::Ordering::Less) => (),
            Some(_) => {
                return Err(BadParameter {
                    msg: String::from("psurvival must be 0 <= p < 1.0"),
                });
            }
            None => (),
        }

        Ok(())
    }
}

#[derive(Copy, Clone)]
struct Diploid {
    node0: tskit::NodeId,
    node1: tskit::NodeId,
}

struct Parents {
    index: usize,
    parent0: Diploid,
    parent1: Diploid,
}

fn death_and_parents(
    alive: &[Diploid],
    params: &SimParams,
    parents: &mut Vec<Parents>,
    rng: &mut StdRng,
) {
    let random_parents = Uniform::new(0_usize, params.popsize as usize);
    for index in 0..alive.len() {
        let x: f64 = rng.gen();
        match x.partial_cmp(&params.psurvival) {
            Some(std::cmp::Ordering::Greater) => {
                let parent0 = alive[rng.sample(random_parents)];
                let parent1 = alive[rng.sample(random_parents)];
                parents.push(Parents {
                    index,
                    parent0,
                    parent1,
                });
            }
            Some(_) => (),
            None => (),
        }
    }
}

fn mendel(pnodes: &mut (tskit::NodeId, tskit::NodeId), rng: &mut StdRng) {
    let x: f64 = rng.gen();
    match x.partial_cmp(&0.5) {
        Some(std::cmp::Ordering::Less) => {
            std::mem::swap(&mut pnodes.0, &mut pnodes.1);
        }
        Some(_) => (),
        None => panic!("Unexpected None"),
    }
}

fn crossover_and_record_edges_details(
    parent: Diploid,
    offspring_node: tskit::NodeId,
    params: &SimParams,
    tables: &mut tskit::TableCollection,
    rng: &mut StdRng,
) {
    let mut pnodes = (parent.node0, parent.node1);
    mendel(&mut pnodes, rng);

    if params.xovers == 0.0 {
        match tables.add_edge(0., tables.sequence_length(), pnodes.0, offspring_node) {
            Ok(_) => (),
            Err(e) => panic!("{}", e),
        }
    } else {
        let exp = match Exp::new(params.xovers / f64::from(tables.sequence_length())) {
            Ok(e) => e,
            Err(e) => panic!("{}", e),
        };
        let mut current_pos = 0.0;
        loop {
            let next_length = rng.sample(exp);
            match (current_pos + next_length).partial_cmp(&tables.sequence_length()) {
                Some(std::cmp::Ordering::Less) => {
                    match tables.add_edge(
                        current_pos,
                        current_pos + next_length,
                        pnodes.0,
                        offspring_node,
                    ) {
                        Ok(_) => (),
                        Err(e) => panic!("{}", e),
                    }
                    std::mem::swap(&mut pnodes.0, &mut pnodes.1);
                    current_pos += next_length;
                }
                Some(_) => {
                    match tables.add_edge(
                        current_pos,
                        tables.sequence_length(),
                        pnodes.0,
                        offspring_node,
                    ) {
                        Ok(_) => (),
                        Err(e) => panic!("{}", e),
                    }
                    break;
                }
                None => panic!("Unexpected None"),
            }
        }
    }
}

fn crossover_and_record_edges(
    parents: &Parents,
    offspring_nodes: (tskit::NodeId, tskit::NodeId),
    params: &SimParams,
    tables: &mut tskit::TableCollection,
    rng: &mut StdRng,
) {
    crossover_and_record_edges_details(parents.parent0, offspring_nodes.0, params, tables, rng);
    crossover_and_record_edges_details(parents.parent1, offspring_nodes.1, params, tables, rng);
}

fn births(
    parents: &[Parents],
    params: &SimParams,
    birth_time: u32,
    tables: &mut tskit::TableCollection,
    alive: &mut [Diploid],
    rng: &mut StdRng,
) {
    for p in parents {
        // Register the two nodes for our offspring
        let node0 = match tables.add_node(
            0,                         // flags
            birth_time as f64,         // time
            tskit::PopulationId::NULL, // population
            // individual
            tskit::IndividualId::NULL,
        ) {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };
        let node1 = match tables.add_node(
            0,
            birth_time as f64,
            tskit::PopulationId::NULL,
            tskit::IndividualId::NULL,
        ) {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };

        // Replace a dead individual
        // with our newborn.
        alive[p.index] = Diploid { node0, node1 };

        crossover_and_record_edges(p, (node0, node1), params, tables, rng);
    }
}

fn rotate_edge_table(mid: usize, tables: &mut tskit::TableCollection) {
    // NOTE: using unsafe here because we don't have
    // a rust API yet.
    let num_edges: usize = tables.edges().num_rows().try_into().unwrap();
    let parent =
        unsafe { std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.parent, num_edges) };
    let child =
        unsafe { std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.child, num_edges) };
    let left =
        unsafe { std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.left, num_edges) };
    let right =
        unsafe { std::slice::from_raw_parts_mut((*tables.as_mut_ptr()).edges.right, num_edges) };
    parent.rotate_left(mid);
    child.rotate_left(mid);
    left.rotate_left(mid);
    right.rotate_left(mid);
}

fn simplify(
    bookmark: &tskit::types::Bookmark,
    alive: &mut [Diploid],
    tables: &mut tskit::TableCollection,
) {
    let mut samples = vec![];
    for a in alive.iter() {
        assert!(a.node0 != a.node1);
        samples.push(a.node0);
        samples.push(a.node1);
    }

    match tables.sort(bookmark, tskit::TableSortOptions::default()) {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    }

    if bookmark.offsets.edges > 0 {
        let mid: usize = bookmark.offsets.edges.try_into().unwrap();
        rotate_edge_table(mid, tables);
    }

    match tables.simplify(
        &samples,
        tskit::SimplificationOptions::KEEP_INPUT_ROOTS,
        true,
    ) {
        Ok(x) => match x {
            Some(idmap) => {
                for a in alive.iter_mut() {
                    a.node0 = idmap[usize::try_from(a.node0).unwrap()];
                    assert!(a.node0 != tskit::NodeId::NULL);
                    a.node1 = idmap[usize::try_from(a.node1).unwrap()];
                    assert!(a.node1 != tskit::NodeId::NULL);
                }
            }
            None => panic!("Unexpected None"),
        },
        Err(e) => panic!("{}", e),
    };
}

fn update_bookmark(
    alive: &[Diploid],
    tables: &mut tskit::TableCollection,
    bookmark: &mut tskit::types::Bookmark,
) -> Result<(), tskit::TskitError> {
    // get min/max time of alive nodes
    let mut most_recent_birth_time: f64 = f64::MAX;
    let mut most_ancient_birth_time: f64 = f64::MIN;

    {
        let nodes = tables.nodes();
        for a in alive {
            for node in [a.node0, a.node1] {
                match nodes.time(node) {
                    Ok(time) => {
                        most_recent_birth_time = if time < most_recent_birth_time {
                            time.into()
                        } else {
                            most_recent_birth_time
                        };
                        most_ancient_birth_time = if time > most_ancient_birth_time {
                            time.into()
                        } else {
                            most_ancient_birth_time
                        };
                    }
                    Err(e) => return Err(e),
                }
            }
        }
    }

    // All alive nodes born at same time.
    if most_ancient_birth_time == most_recent_birth_time {
        bookmark.offsets.edges = tables.edges().num_rows().into();
    } else {
        // We have non-overlapping generations:
        // * Find the last node born at <= the max time
        // * Rotate the edge table there
        // * Set the bookmark to include the rotated nodes

        // NOTE: we dip into unsafe here because we
        // don't yet have direct API support for these ops.

        let num_nodes: usize = tables.nodes().num_rows().try_into().unwrap();
        let time = unsafe { std::slice::from_raw_parts((*tables.as_ptr()).nodes.time, num_nodes) };
        match time
            .iter()
            .enumerate()
            .find(|(_index, time)| **time > most_ancient_birth_time)
        {
            Some((index, _time)) => {
                rotate_edge_table(index, tables);
                let num_edges: usize = tables.edges().num_rows().try_into().unwrap();
                bookmark.offsets.edges = (num_edges - index).try_into().unwrap();
            }
            None => bookmark.offsets.edges = 0,
        }
    }

    Ok(())
}

fn runsim(params: &SimParams) -> tskit::TableCollection {
    let mut tables = match tskit::TableCollection::new(params.genome_length) {
        Ok(x) => x,
        Err(e) => panic!("{}", e),
    };

    let mut rng = StdRng::seed_from_u64(params.seed);

    let mut alive: Vec<Diploid> = vec![];
    for _ in 0..params.popsize {
        let node0 = match tables.add_node(
            0,
            params.nsteps as f64,
            tskit::PopulationId::NULL,
            tskit::IndividualId::NULL,
        ) {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };
        let node1 = match tables.add_node(
            0,
            params.nsteps as f64,
            tskit::PopulationId::NULL,
            tskit::IndividualId::NULL,
        ) {
            Ok(x) => x,
            Err(e) => panic!("{}", e),
        };
        alive.push(Diploid { node0, node1 });
    }

    let mut parents: Vec<Parents> = vec![];
    let mut simplified: bool = false;

    let mut bookmark = tskit::types::Bookmark::new();
    for step in (0..params.nsteps).rev() {
        parents.clear();
        death_and_parents(&alive, params, &mut parents, &mut rng);
        births(&parents, params, step, &mut tables, &mut alive, &mut rng);
        let remainder = step % params.simplification_interval;
        match step < params.nsteps && remainder == 0 {
            true => {
                simplify(&bookmark, &mut alive, &mut tables);
                simplified = true;
                update_bookmark(&alive, &mut tables, &mut bookmark).unwrap();
            }
            false => simplified = false,
        }
    }

    if !simplified {
        simplify(&bookmark, &mut alive, &mut tables);
    }

    tables
}

fn main() {
    let params = SimParams::parse();
    params.validate().unwrap();

    let tables = runsim(&params);
    let treeseq = tables
        .tree_sequence(tskit::TreeSequenceFlags::BUILD_INDEXES)
        .unwrap();
    treeseq
        .dump(&params.treefile, tskit::TableOutputOptions::default())
        .unwrap();
}

#[test]
#[should_panic]
fn test_bad_genome_length() {
    let mut params = SimParams::default();
    params.genome_length = -1.0;
    let _tables = runsim(&params);
}

#[test]
fn test_nonoverlapping_generations() {
    let mut params = SimParams::default();
    params.nsteps = 500;
    params.xovers = 1e-3;
    params.validate().unwrap();
    runsim(&params);
}

#[test]
fn test_overlapping_generations() {
    let mut params = SimParams::default();
    params.nsteps = 100;
    params.xovers = 1e-3;
    params.psurvival = 0.25;
    params.seed = 616161;
    params.validate().unwrap();
    runsim(&params);
}
