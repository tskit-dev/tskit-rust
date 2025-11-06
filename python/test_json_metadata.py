import tskit
import numpy as np


def setup_ts_without_schema():
    ts = tskit.TreeSequence.load("with_json_metadata.trees")
    return ts


def setup_ts_with_schema():
    ts = setup_ts_without_schema()
    tables = ts.tables
    tables.individuals.metadata_schema = tskit.metadata.MetadataSchema(
        {
            "codec": "json",
            "type": "object",
            "name": "Individual metadata",
            "properties": {"name": {"type": "string"},
                           "phenotypes": {"type": "array"}},
            "additionalProperties": False,
        })
    tables.mutations.metadata_schema = tskit.metadata.MetadataSchema(
        {
            "codec": "json",
            "type": "object",
            "name": "Individual metadata",
            "properties": {"effect_size": {"type": "number"},
                           "dominance": {"type": "number"}},
            "additionalProperties": False,
        })
    return tables.tree_sequence()


def test_individual_metadata():
    # NOTE: the assertions here rely on knowing
    # what examples/json_metadata.rs put into the
    # metadata!
    ts = setup_ts_with_schema()
    md = ts.individual(0).metadata
    assert md["name"] == "Jerome"
    assert md["phenotypes"] == [0, 1, 2, 0]


def test_individual_metadata_without_schema():
    # NOTE: the assertions here rely on knowing
    # what examples/json_metadata.rs put into the
    # metadata!
    ts = setup_ts_without_schema()
    md = eval(ts.individual(0).metadata)
    assert md["name"] == "Jerome"
    assert md["phenotypes"] == [0, 1, 2, 0]


def test_mutation_metadata():
    # NOTE: the assertions here rely on knowing
    # what examples/json_metadata.rs put into the
    # metadata!
    ts = setup_ts_with_schema()
    md = ts.mutation(0).metadata
    assert np.isclose(md["effect_size"], -1e-3)
    assert np.isclose(md["dominance"], 0.1)


def test_mutation_metadata_without_schema():
    # NOTE: the assertions here rely on knowing
    # what examples/json_metadata.rs put into the
    # metadata!
    ts = setup_ts_without_schema()
    md = eval(ts.mutation(0).metadata)
    assert np.isclose(md["effect_size"], -1e-3)
    assert np.isclose(md["dominance"], 0.1)
