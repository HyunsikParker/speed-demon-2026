# Track A: Ingest

An exact hourly HTTP-log aggregator for the [Track A specification](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/a-ingest.md).

## Approach

The program reads `requests.log` through a reusable byte buffer. The first 13
bytes of each fixed-width UTC timestamp identify its hour, so grouping does not
require a date parser. Hour prefixes and endpoint byte strings are interned into
integer IDs as they appear. Each endpoint has 72 group slots, matching the
specified time span; input order does not determine output order.

Each group keeps its latencies as `u32` values and its error count as `u64`.
Duplicate requests are retained. A status beginning with `5` is an error because
the input grammar restricts statuses to 100–599. After reading the file, the
program sorts the observed hours and endpoints by bytes, then sorts each nonempty
group's latencies. Nearest-rank percentiles and error parts per million use
integer arithmetic. Empty input produces an empty `summary.txt`.

All parsing and aggregation happen at runtime on one thread. The image contains
no data, cached results or input-derived index. The starter Python files remain
unchanged as reference material; the container runs the Rust binary.

## Build and run

The Dockerfile builds offline with digest-pinned images from the official
allowlist. It reads `/data/requests.log` and writes only `/out/summary.txt`.
For a native development run, use `cargo build --release --offline`, then
`target/release/ingest DATA_DIRECTORY OUTPUT_DIRECTORY`. The output directory
must already exist.

## Dependencies

Rust standard library only; no external crates or analytics libraries.
Codex developed the implementation and validation. [PERF.md](PERF.md) records
the measurements and their limits.

## With another 24 hours

Profile hash lookups and per-group sorting before adding parallel parsing.
Parallel workers would need to merge duplicate keys and preserve exact counts
and output order; that change needs a separate correctness check and measurement.
