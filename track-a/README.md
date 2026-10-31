# Track A: Ingest

An exact hourly HTTP-log aggregator for the [Track A
specification](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/a-ingest.md).

## Approach

Up to six workers read separate byte ranges of `requests.log` through reusable
buffers. A range owns each line whose first byte falls within it. Workers
inspect the preceding byte and skip any partial starting line, then finish the
last line they own. This preserves requests that cross range boundaries
without copying the whole input into memory. It does not assume timestamp
order.

The first 13 bytes of each fixed-width UTC timestamp identify its hour, so
grouping does not require a date parser. Hour prefixes are interned into
integer IDs in each worker. A word-wise scan finds the endpoint boundary and
computes its fingerprint in one pass. Each fingerprint maps to a bucket of
endpoint IDs; every match compares the complete endpoint bytes, so a hash
collision cannot merge different endpoints. Each endpoint starts with an empty
group vector. The vector grows through the highest local hour ID used for that
endpoint. Missing group slots and empty merge sources are skipped; input order
does not determine output order.

Each group keeps its latencies as `u32` values and its error count as `u64`.
Duplicate requests are retained. A status beginning with `5` is an error
because the input grammar restricts statuses to 100–599. The main thread
merges the workers' groups by their complete hour and endpoint keys, adds
error counts and combines latency vectors. A vector can move directly into an
empty destination; overlapping groups append all their values, including
duplicates. The program then sorts the observed hours and endpoints by bytes.
Output workers receive contiguous endpoint blocks, sort each nonempty group's
latencies and format one hour at a time. A zero-capacity channel transfers
each hour buffer to the main thread, which writes in hour and endpoint-block
order. Completed latency arrays are released. Nearest-rank percentiles and
error parts per million use integer arithmetic. Empty input produces an empty
`summary.txt`.

All parsing and aggregation happen at runtime. Reading and group formatting
use scoped workers; merging and final ordered writes are serial. The image
contains no data, cached results or input-derived index. The starter Python
files remain unchanged as reference material; the container runs the Rust
binary.

## Build and run

The Dockerfile builds offline with digest-pinned images from the official
allowlist. It reads `/data/requests.log` and writes only `/out/summary.txt`.
For a native development run, use `cargo build --release --offline`, then
`target/release/ingest DATA_DIRECTORY OUTPUT_DIRECTORY`. The output directory
must already exist. `SD_WORKERS` selects 1–12 readers, capped by available
parallelism; the default is six. The same merge and output rules apply at one
reader.

## Dependencies

Rust standard library only; no external crates or analytics libraries. Codex
developed the implementation and validation. [PERF.md](PERF.md) records the
measurements and their limits.

## Limits

Group vectors still include empty slots before the highest observed local hour
ID, and vector capacity can exceed its length. Memory therefore depends on how
hours and endpoints are distributed across ranges. Merging and final ordered
writes remain serial. Output buffering is bounded by the workers' current hour
buffers and the buffer being written. Each group is emitted once; exact
latency values and duplicate counts are retained until its percentiles and row
have been produced.


## With another 24 hours

Measure the serial merge's contribution and the distribution of latency-vector
capacity across endpoint/hour groups. A useful next candidate would reduce that
merge work while preserving every latency value and duplicate. Endpoint storage
layouts already missed their fixed efficiency gates; they are not an untested
shortcut. Any new mechanism needs a separate whole-program comparison and exact
output checks before adoption.
