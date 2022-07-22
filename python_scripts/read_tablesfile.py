import tskit
import sys

for f in sys.argv[1:]:
    tables = tskit.TableCollection.load(f)
    for pop in tables.populations:
        print(pop)
