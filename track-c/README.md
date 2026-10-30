# Track C: Join

An exact CSV parser, region join and country aggregator for the
[Track C specification](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/c-join.md).

## Approach

The reader appends physical lines to a reusable record buffer until quote parity
is even. Escaped quote pairs contribute two quotes, so they do not change that
parity. A newline inside a quoted field stays in the record. The parser removes
one BOM only at a record boundary, then unescapes name bytes and counts decoded
note bytes. It preserves empty values, trailing spaces, embedded newlines and
literal BOM bytes inside text. Notes are counted without storing their contents.

The region table maps integer IDs directly to compact country IDs. Missing IDs,
including holes within the table's range, map to `_UNMATCHED`. Names are interned
globally as exact byte strings. Each country keeps a dynamically growing bitset
of those IDs and a distinct-name counter. A name first seen in that country
sets one bit and increments the counter; repeats leave it unchanged. This
avoids a second hash lookup and another copy of the name. The bitset grows from
the IDs actually observed; it does not assume a fixed name-pool size.
No input-derived table is shipped: these structures are built from the input on
each run.

Counts and ID totals use 64-bit integers. Money is parsed and aggregated as
signed 64-bit cents, including values between -1.00 and 0.00. Output contains only
countries with records, sorted by their byte keys, with exact two-digit money
formatting. The implementation uses one thread.

## Build and run

The Dockerfile builds offline with digest-pinned images from the official
allowlist. It reads `/data/records.csv` and `/data/regions.csv`, then writes only
`/out/aggregates.txt`. For a native run, use `cargo build --release --offline`,
then `target/release/join DATA_DIRECTORY OUTPUT_DIRECTORY`. The output directory
must already exist. The starter Python files are unchanged reference material;
the container runs the Rust binary.

## Dependencies

Rust standard library only; no CSV crate, dataframe or SQL engine.
Codex developed the implementation and validation. [PERF.md](PERF.md) records
the measurements and their limits.

## With another 24 hours

Profile CSV decoding and the global name interner separately. The country
bitsets reserve space up to their highest observed global name ID, even when
a country contains few distinct names. A sparse representation is worth
comparing on inputs with many global names and sparse country membership.
