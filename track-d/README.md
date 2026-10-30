# Track D: Traverse

An exact directed shortest-path solver for the [Track D specification](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/d-traverse.md).

## Approach

The parser builds compressed sparse row (CSR) adjacency arrays. Each edge keeps
its destination and weight as 32-bit integers; distances use 64 bits. Parallel
edges, duplicate edges, self-loops and zero weights retain their specified meaning.

Each query runs forward Dijkstra with a binary heap. A worker reuses its distance
array between queries, using an epoch array to identify current values instead of
clearing every distance. By default, up to six workers take independent queries from a shared
index; answers are written in the original query order after all workers finish.

Same-node queries return zero before any degree checks. For distinct endpoints,
a source with no outgoing edges or a target with no incoming edges is unreachable.
All other queries use the directed search. Different strongly connected components
do not imply unreachability, so this version makes no SCC-based rejection.

Parsing, graph construction and all searches happen at runtime. The image contains
no datasets, cached answers or input-derived index. The unchanged Python files
from the starter remain as reference material; the container runs the Rust binary.

## Build and run

The Dockerfile uses digest-pinned images from the official allowlist and builds
with Cargo offline. The runtime reads `/data/graph.txt` and `/data/queries.txt`,
then writes only `/out/distances.txt`.

For a native development run, build with `cargo build --release --offline`, then
run `target/release/traverse DATA_DIRECTORY OUTPUT_DIRECTORY`. The output directory
must exist. `SD_WORKERS` can select 1–12 workers, capped by available parallelism
and the query count; the default is six.

## Dependencies and assistance

Rust standard library only. No external crates or graph-algorithm libraries.
The implementation and validation were developed with Codex. Measurements and
their limits are recorded in [PERF.md](PERF.md).

## Next work

Measure the worker-count trade-off on the scoring machine before changing the
default. A separately checked bidirectional search may reduce explored vertices;
its stopping rule and zero-weight cases need their own correctness tests.
