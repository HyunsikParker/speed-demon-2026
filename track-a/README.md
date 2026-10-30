# Track A: Ingest

An exact hourly HTTP-log aggregator for the [Track A specification](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/a-ingest.md).

## Approach

Up to six workers read separate byte ranges of `requests.log` through reusable
buffers. A range owns each line whose first byte falls within it. Workers inspect
the preceding byte and skip any partial starting line, then finish the last line
they own. This preserves requests that cross range boundaries without copying
the whole input into memory. It does not assume timestamp order.

The first 13
bytes of each fixed-width UTC timestamp identify its hour, so grouping does not
require a date parser. Hour prefixes and endpoint byte strings are interned into
integer IDs separately in each worker. Each endpoint has 72 group slots, matching the
specified time span; input order does not determine output order.

Each group keeps its latencies as `u32` values and its error count as `u64`.
Duplicate requests are retained. A status beginning with `5` is an error because
the input grammar restricts statuses to 100–599. The main thread merges the
workers' groups by their complete hour and endpoint keys, adds error counts and
combines latency vectors. A vector can move directly into an empty destination;
overlapping groups append all their values, including duplicates. The program
then sorts the observed hours and endpoints by bytes, followed by each nonempty
group's latencies. Nearest-rank percentiles and error parts per million use
integer arithmetic. Empty input produces an empty `summary.txt`.

All parsing and aggregation happen at runtime. Parsing uses independent workers;
merging, sorting and output are serial. The image contains
no data, cached results or input-derived index. The starter Python files remain
unchanged as reference material; the container runs the Rust binary.

## Build and run

The Dockerfile builds offline with digest-pinned images from the official
allowlist. It reads `/data/requests.log` and writes only `/out/summary.txt`.
For a native development run, use `cargo build --release --offline`, then
`target/release/ingest DATA_DIRECTORY OUTPUT_DIRECTORY`. The output directory
must already exist. `SD_WORKERS` selects 1–12 readers, capped by available
parallelism; the default is six. The same merge and output rules apply at one reader.

## Dependencies

Rust standard library only; no external crates or analytics libraries.
Codex developed the implementation and validation. [PERF.md](PERF.md) records
the measurements and their limits.

## Next work

Per-worker interners and 72-slot group arrays increase peak memory, especially on
the sample. A future change should measure that replicated state and the merge
phase before reducing allocations. Exact latency values, duplicate counts and
byte-ordered output must remain unchanged.
