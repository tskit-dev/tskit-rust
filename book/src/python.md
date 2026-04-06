# Data transfer to Python

The most straightforward method is to write data to a file.
To do so, call `TableCollction::dump` or `TreeSequence::dump` as appropriate.

It is also possible to transfer data to a Python tree sequence without making a copy.
At the time of this writing, an experimental repository to do so is [here](https://github.com/molpopgen/tskit2tskit).
