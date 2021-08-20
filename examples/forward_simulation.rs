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
use clap::{value_t, App, Arg};
use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;
use rand_distr::{Exp, Uniform};

struct SimParams {
    pub popsize: u32,
    pub nsteps: u32,
    pub xovers: f64,
    pub psurvival: f64,
    pub genome_length: f64,
    pub simplification_interval: u32,
    pub treefile: String,
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
    fn new() -> Self {
        let mut params = SimParams::default();

        let matches = App::new("forward_simulation")
            .arg(
                Arg::with_name("popsize")
                    .short("N")
                    .long("popsize")
                    .help("Diploid population size. Default = 1,000.")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("nsteps")
                    .short("n")
                    .long("nsteps")
                    .help("Number of birth steps to simulate. For non-overlapping generations, this is the number of generations to simulate. Default = 1,000.")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("xovers")
                    .short("x")
                    .long("xovers")
                    .help("Mean number of crossovers per meiosis. The number of crossovers is Poisson-distributed with this value. Default = 0.0.")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("genome_length")
                    .short("L")
                    .long("genome_length")
                    .help("Genome length (continuous units).  Default = 1e6.")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("simplification_interval")
                    .short("s")
                    .long("simplify")
                    .help("Number of birth steps between simplifications. Default = 100.")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("treefile")
                    .short("t")
                    .long("treefile")
                    .help("Name of output file. The format is a tskit \"trees\" file. Default = \"treefile.trees\".")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("seed")
                    .short("S")
                    .long("seed")
                    .help("Random number seed. Default = 0.")
                    .takes_value(true),
            )
            .arg(
                Arg::with_name("psurvival")
                    .short("P")
                    .long("psurvival")
                    .help("Survival probability. A value of 0.0 is the Wright-Fisher model of non-overlapping generations.  Values must b 0.0 <= p < 1.0.  Default = 0.0.")
                    .takes_value(true),
            )
            .get_matches();

        params.popsize = value_t!(matches.value_of("popsize"), u32).unwrap_or(params.popsize);
        params.nsteps = value_t!(matches.value_of("nsteps"), u32).unwrap_or(params.nsteps);
        params.xovers = value_t!(matches.value_of("xovers"), f64).unwrap_or(params.xovers);
        params.genome_length =
            value_t!(matches.value_of("genome_length"), f64).unwrap_or(params.genome_length);
        params.simplification_interval = value_t!(matches.value_of("simplification_interval"), u32)
            .unwrap_or(params.simplification_interval);
        params.seed = value_t!(matches.value_of("seed"), u64).unwrap_or(params.seed);
        params.psurvival = value_t!(matches.value_of("psurvival"), f64).unwrap_or(params.psurvival);
        params.treefile = value_t!(matches.value_of("treefile"), String).unwrap_or(params.treefile);

        params
    }

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
        let exp = match Exp::new(params.xovers / tables.sequence_length()) {
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

fn simplify(alive: &mut [Diploid], tables: &mut tskit::TableCollection) {
    let mut samples = vec![];
    for a in alive.iter() {
        assert!(a.node0 != a.node1);
        samples.push(a.node0);
        samples.push(a.node1);
    }

    match tables.full_sort(tskit::TableSortOptions::default()) {
        Ok(_) => (),
        Err(e) => panic!("{}", e),
    }

    match tables.simplify(
        &samples,
        tskit::SimplificationOptions::KEEP_INPUT_ROOTS,
        true,
    ) {
        Ok(x) => match x {
            Some(idmap) => {
                for a in alive.iter_mut() {
                    a.node0 = idmap[usize::from(a.node0)];
                    assert!(a.node0 != tskit::NodeId::NULL);
                    a.node1 = idmap[usize::from(a.node1)];
                    assert!(a.node1 != tskit::NodeId::NULL);
                }
            }
            None => panic!("Unexpected None"),
        },
        Err(e) => panic!("{}", e),
    };
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

    for step in (0..params.nsteps).rev() {
        parents.clear();
        death_and_parents(&alive, params, &mut parents, &mut rng);
        births(&parents, params, step, &mut tables, &mut alive, &mut rng);
        let remainder = step % params.simplification_interval;
        match step < params.nsteps && remainder == 0 {
            true => {
                simplify(&mut alive, &mut tables);
                simplified = true;
            }
            false => simplified = false,
        }
    }

    if !simplified {
        simplify(&mut alive, &mut tables);
    }

    tables
}

fn main() {
    let params = SimParams::new();
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
    let mut params = SimParams::new();
    params.genome_length = -1.0;
    let _tables = runsim(&params);
}

#[test]
fn test_nonoverlapping_generations() {
    let mut params = SimParams::new();
    params.nsteps = 100;
    params.xovers = 1e-3;
    params.validate().unwrap();
    runsim(&params);
}

#[test]
fn test_overlapping_generations() {
    let mut params = SimParams::new();
    params.nsteps = 100;
    params.xovers = 1e-3;
    params.psurvival = 0.25;
    params.seed = 616161;
    params.validate().unwrap();
    runsim(&params);
}
