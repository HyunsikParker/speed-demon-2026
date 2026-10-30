# Track B: Index

An exact Boolean and phrase-query engine for the
[Track B specification](https://github.com/hackathon-raptors/speed-demon-2026/blob/main/specs/b-index.md).

## Approach

The program reads the queries first and interns their terms. It streams the
corpus once, retaining postings only for those terms. Each posting list contains
sorted, unique document IDs. Terms used in a phrase also retain sorted 64-bit
pairs of document ID and token position. Unindexed tokens still advance the
position counter, so leaving a term out of the index cannot create adjacency.

Tokenization lowercases ASCII bytes and splits on every byte outside `a`–`z`
and `0`–`9`. It performs no Unicode normalization or case folding. Numeric tokens
remain byte strings: `007` and `7` are distinct.

Boolean operations merge sorted lists in the order written. `NOT` is a binary
set difference; `AND` has no precedence over `OR`. An intersection with a large
list-size imbalance binary-searches the larger list for each smaller-list ID.

For a phrase, the shortest positional list supplies anchor occurrences. The
program checks each other term at its exact offset within the same document.
A matching document is emitted once even if it contains several occurrences.
This preserves repeated words and phrases across punctuation while preventing
matches across document boundaries.

Answers are already sorted. The program writes the count, a 64-bit ID sum and
at most the first ten IDs for each query, retaining duplicate queries in order.
It uses one thread. All indexes are built from the supplied inputs during the
timed run; the image contains no corpus, cached answers or prebuilt index.

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

Measure indexing and query time separately before replacing the posting layout.
Frequent terms may benefit from bitmaps, but sparse queries would pay for scans
of empty words. Compare that tradeoff with the sorted-list version while keeping
phrase positions and left-to-right Boolean semantics unchanged.
