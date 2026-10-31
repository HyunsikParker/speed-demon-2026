# Track B: Index

An exact Boolean and phrase-query engine for the
[Track B specification](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/b-index.md).

## Approach

The program reads the queries first and interns their terms. Up to six workers
share that read-only dictionary and process disjoint document ranges. Boolean
queries use blocks of 512 documents. A worker allocates a bitmap row only when
a queried Boolean term occurs in that block. Phrase queries become a shared
token-pattern automaton; no positional posting lists are stored.

The dictionary uses fixed-size slots holding the first eight bytes, byte length
and original term ID. Short keys are exact in the slot; longer keys also require
full byte equality. Linear probing resolves collisions, including shared
prefixes, and the table is built from the supplied queries during the run.

Each query stores its terms and operators inline, using the specification's
bounds of six Boolean terms or eight phrase terms. Term IDs are checked before
conversion to 32 bits. The actual length excludes padding from evaluation and
phrase construction. These records retain the original sequential query order.

Tokenization lowercases ASCII bytes and splits on every byte outside `a`–`z`
and `0`–`9`. It performs no Unicode normalization or case folding. Numeric tokens
remain byte strings: `007` and `7` are distinct.

Boolean operations combine eight-word bitmaps in the order written. `NOT` is
a binary set difference; `AND` has no precedence over `OR`. Missing rows are
zero masks. A conservative presence check can skip a query that cannot match;
it never uses a right-hand `NOT` term's presence to conclude that a result is
empty. After each block, workers accumulate counts, ID sums and the smallest
ten IDs, then clear the active bitmap rows. Summary storage is allocated on
the first match.

Phrase matching uses an independently implemented Aho–Corasick automaton over
token IDs. It is built from the supplied queries during the timed run. Failure
links retain suffix matches, and output lists report all overlapping patterns.
Identical phrase queries share one pattern ID but keep separate output lines.
Each worker resets the automaton at every document boundary. A token absent from
the query dictionary also resets it, so omitted terms cannot create adjacency.
Repeated matches of one phrase in a document contribute only one document ID.
Those matches use sparse document-ID lists and are summarized after the shard.
The [algorithm's all-overlap semantics](https://docs.rs/aho-corasick/latest/aho_corasick/)
are relevant here; this implementation does not use that crate or its code.

Each worker owns a contiguous byte range. It aligns the beginning to the next
complete document and finishes any document that begins before its range ends.
Checking the byte before a boundary distinguishes an exact line start from a
partial record. Each document is therefore assigned once, including when a
long document crosses several ranges. Workers stream from separate file handles;
the program does not copy the corpus into memory.

Every worker evaluates all queries against its own document set. Restricting a
set to disjoint documents commutes with union, intersection and binary difference,
so left-to-right Boolean evaluation remains exact. Phrase matches stay inside
one document. Workers return only counts, 64-bit ID sums and up to ten smallest
matching IDs per query. Counts and sums add; concatenating the sorted summaries
in document-range order and taking ten gives the global ten smallest IDs.
Duplicate queries retain their input order.

The default is six workers. `SD_WORKERS` selects a value clamped to 1–6. All
indexes are built from the supplied inputs during the timed run; the image
contains no corpus, cached answers or prebuilt index.

## Build and run

The Dockerfile builds offline with digest-pinned images from the official
allowlist. It reads `/data/docs.txt` and `/data/queries.txt`, then writes only
`/out/results.txt`. For a native run, use `cargo build --release --offline`, then
`target/release/index DATA_DIRECTORY OUTPUT_DIRECTORY`. The output directory
must already exist. The starter Python files remain unchanged as reference
material; the container runs the Rust binary.

## Dependencies

Rust standard library only; no search engine, database or external crate.
Codex developed the implementation and validation. [PERF.md](PERF.md) records
the measurements and their limits.

## With another 24 hours

Compare the full-size live and held-out measurements, then profile the
remaining dictionary and bitmap work on the accepted source. Any further
change should preserve the exact tokenizer, query order and collision checks,
and pass the same independent oracle and whole-program efficiency gates.
