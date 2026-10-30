# Track D: Traverse

An exact directed shortest-path solver for the [Track D specification](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/d-traverse.md).

## Approach

The parser builds forward and reverse compressed sparse row (CSR) adjacency
arrays. After parsing the edge triples, it releases the input byte buffer before
building the two CSR directions. Each edge keeps its destination and weight as
32-bit integers; queue keys, path arithmetic and output use 64 bits. Reversing
an edge preserves its weight rather than borrowing the
weight of a separate edge in the opposite direction. Parallel edges, duplicate
edges, self-loops and zero weights retain their specified meaning.

Each query alternates between a forward Dijkstra search from the source and a
reverse search from the target. A vertex reached by both searches supplies an
upper bound from a real directed path. The search stops only when the sum of
the two minimum pending distances cannot improve that bound, not at the first
meeting. Stale heap entries are removed before testing this stopping condition.

Each direction uses a 65-bucket minimum radix heap. Buckets group 64-bit distances
by their highest differing bit from the last minimum. When bucket zero empties,
the next nonempty bucket establishes a new minimum and redistributes its entries.
Nonnegative edge weights ensure that newly queued distances never fall below
that minimum. Equal keys may be returned in any order, and each new query resets
the queue to zero. The buckets retain their allocated capacity between queries.

Each search stores distances below 4,294,967,294 in a `u32` array. The two
remaining values mark an unvisited node or an exact `u64` value in a hash map.
Large distances are never truncated. If a shorter path fits inline, its old
map entry is removed. A worker reuses two such arrays, resets only touched
vertices and clears the overflow maps for each new query. By default, up to
six workers take independent queries from a
shared index; answers are written in input order after every worker finishes.

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

The [radix-heap documentation](https://docs.rs/radix-heap/latest/radix_heap/)
describes the monotone-queue concept with a maximum queue. This implementation
uses the corresponding minimum-key invariant and was written independently;
it does not use the crate or copy its source.

## Limits

The overflow maps can grow on inputs with many large distances. Inline storage
adds branches to distance access, and graph construction still needs parsed
triples alongside CSR storage. The performance log measures the combined
working-set change; it does not isolate either part's contribution.
