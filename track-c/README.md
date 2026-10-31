# Track C: Join

An exact CSV parser, region join and country aggregator for the
[Track C specification](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/c-join.md).

## Approach

The input is memory-mapped read-only. Six byte chunks first count quote parity
in parallel. On x86-64 CPUs with AVX2, each worker XORs 32-byte quote masks and
reduces the result to one bit; other CPUs and short tails use scalar code.
Prefix XOR gives the quote state at each of the five interior chunk boundaries.
A forward scan then moves each cut to the next complete record. Escaped quote
pairs, quoted newlines and records spanning several chunks retain their meaning.

Six parsing workers borrow the resulting disjoint ranges from the mapping.
Within each range, quote-aware scanning forms batches targeting 64 KiB, with
oversized records kept intact. No input buffers or producer queues are needed.
The mapping stays alive until every worker has finished.

The parser removes one BOM only at a record boundary, then unescapes name bytes
and counts decoded note bytes without storing the notes. It preserves empty
values, trailing spaces, embedded newlines and literal BOM bytes inside text.

The region table maps integer IDs directly to compact country IDs. Missing IDs,
including holes within the table's range, map to `_UNMATCHED`. A shared map,
partitioned across 64 mutex-protected shards, stores exact name bytes and their
country-membership bitsets. Each worker groups a batch's name requests by shard
and locks each nonempty shard once. The first `(name, country)` pair sets a bit
and increments that worker's distinct count while holding the lock. Summing the
worker counts therefore preserves global distinctness across batches.

For tables with at most 256 country groups, including `_UNMATCHED`, names of
up to 16 bytes use inline entries: two words hold the complete name, a separate length distinguishes
zero-padded values, and four words hold the country bits. These requests carry
their keys directly, without copying them into the decoded-name arena. Longer
names and tables with more countries retain the full-byte map and variable-size
bitsets. Both paths preserve exact name identity.

Workers retain their name buffers, decoded-name arenas and request-vector
capacities between batches, clearing their lengths before reuse. Numeric
aggregates remain worker-local until the final merge. All lookup structures are
built from the current input; no input-derived table or precomputed answer is
shipped.

Counts and ID totals use 64-bit integers. Money is parsed and aggregated as
signed 64-bit cents, including values between -1.00 and 0.00. Output contains only
countries with records, sorted by their byte keys, with exact two-digit money
formatting.

## Build and run

The Dockerfile builds offline with digest-pinned images from the official
allowlist. It reads `/data/records.csv` and `/data/regions.csv`, then writes only
`/out/aggregates.txt`. For a native run, use `cargo build --release --offline`,
then `target/release/join DATA_DIRECTORY OUTPUT_DIRECTORY`. The output directory
must already exist. The starter Python files are unchanged reference material;
the container runs the Rust binary.

## Dependencies

Rust standard library and the Linux `mmap`/`munmap` interfaces; no external
crates, CSV library, dataframe or SQL engine. The native binary targets Linux.
Codex developed the implementation and validation. [PERF.md](PERF.md) records
the measurements and their limits.

## With another 24 hours

Measure the current quote-parity scan, range parsing and shared-membership
costs separately, including long-name and large-country-table inputs. The
mapping trial with sequential boundary discovery and the earlier pipeline
profiles remain in PERF.md; their overlapping intervals cannot predict the
savings from this implementation.
