import msprime
import argparse
import sys


def make_parser():
    parser = argparse.ArgumentParser(description="Make some data with msprime")

    parser.add_argument("--nsamples", type=int, help="Number of diploid samples")
    parser.add_argument("--seqlen", type=float, default=1e8, help="sequence length")
    parser.add_argument(
        "--recrate", type=float, default=1e-9, help='Rec rate per "base pair"'
    )
    parser.add_argument("--popsize", type=int, default=10000, help="Population size")
    parser.add_argument(
        "--treefile", type=str, default="treefile.trees", help="Output file name"
    )

    return parser


def main():
    parser = make_parser()
    args = parser.parse_args(sys.argv[1:])

    ts = msprime.sim_ancestry(
        args.nsamples,
        sequence_length=args.seqlen,
        recombination_rate=args.recrate,
        population_size=args.popsize,
    )

    print(ts.num_trees)

    ts.dump(args.treefile)


if __name__ == "__main__":
    main()
