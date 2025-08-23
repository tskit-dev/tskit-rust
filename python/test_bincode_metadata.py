import tskit
import tskit_glue
import numpy as np


def setup_ts_without_schema():
    ts = tskit.TreeSequence.load("with_bincode_metadata.trees")
    return ts


def test_individual_metadata():
    # NOTE: the assertions here rely on knowing
    # what examples/json_metadata.rs put into the
    # metadata!
    ts = setup_ts_without_schema()
    md = tskit_glue.decode_bincode_individual_metadata(ts.individual(0).metadata)
    assert md.name() == "Jerome"
    assert md.phenotypes() == [0, 1, 2, 0]


def test_mutation_metadata():
    # NOTE: the assertions here rely on knowing
    # what examples/json_metadata.rs put into the
    # metadata!
    ts = setup_ts_without_schema()
    md = tskit_glue.decode_bincode_mutation_metadata(ts.mutation(0).metadata)
    assert np.isclose(md.effect_size(), -1e-3)
    assert np.isclose(md.dominance(), 0.1)
