# Track B: performance log

## Live-board history

The first candidate replaces the Python index with a standard-library Rust
query-term index.

| Run | Commit | Result | T (s) | M (bytes) | K | Machine points |
|---|---|---|---:|---:|---:|---:|
| `run-750270f3b3c2` | `bc07ea2` | Correct | 5.433148 | 387,010,560 | 4.136591 | 30.832131 |
| `run-27458ab5e1fd` | `82216f0` | Correct, unchanged B source | 5.915574 | 389,976,064 | 3.958284 | 29.875346 |
| `run-856be110b4b8` | `17838c7` | Correct, unchanged B source | 5.571093 | 386,613,248 | 4.085897 | 30.56437 |

The [official live board](https://speeddemon.dev/board/) reported this run on
2026-10-30, from 20:37:54 to 20:38:00 UTC. Check, build, correctness, timeout and
sandbox gates passed. The board reported S = 174 and D = 1. At 20:39:16 UTC,
all four tracks had valid scores; the overall display was 40.969, rank 40.
B was below C, so it was not included in the best-three geometric mean.
These are live machine points, not final judging or an award.

The next run accompanied D's bidirectional-search update with unchanged B
source. All five gates passed; S = 174 and D = 1 were unchanged. Its latest
29.875346 points were below the prior best 30.832131, which the page
retained. At 20:51:13 UTC the overall display was 42.042, rank 38. This is a
same-source control, not an algorithmic regression or improvement claim.

## Local validation — 2026-10-30

Machine: Intel Core i5-12500H, Ubuntu 24.04 under WSL2, Rust 1.99.0 and Python 3.12.
Both programs ran on the same otherwise idle development machine. Each timed
program read the documents and queries, built its index, evaluated every query
and wrote the complete output. GNU `time` recorded one wall-time and
peak-process-RSS observation per program and scale. These are not official
five-run T or cgroup `memory.peak` M values.

| Public seed 1 data | Python reference wall time | Rust wall time | Reference peak RSS (KiB) | Rust peak RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 1.95 s | 0.18 s | 309,828 | 30,280 |
| Full | 38.00 s | 4.47 s | 2,150,408 | 369,804 |

The full-data pair showed a wall-time speedup of about 8.50×. One pair does not
establish repeatability or predict the board coefficient. Local reference
Python 3.12 also differs from official Python 3.13.

Correctness checks passed:

- The official worked example and all six host edge-case tests, checked against
  literal expectations, the reference and an independent per-document oracle.
- A 100,000-document case with a document-ID sum exceeding 32 bits, repeated
  phrase matches and phrases longer than a document.
- Unindexed intervening tokens and duplicate queries, preserving exact token
  positions and one output line per query.
- Tiny seeds 10 and 11, then reserved seed 12 generated after those checks
  passed. Each contained every host-listed edge-case category and matched both
  independent implementations. The candidate source remained unchanged.
- Published sample/full seed 1 input hashes and exact output hashes.
- Full output from the offline-built image as uid 65534, with no network,
  read-only root, an 8 GB memory limit, no swap and a 1 GB tmpfs. The image was
  30,028,821 bytes. This check did not use the official scoring harness.

Full output SHA-256:
`f86973c210228141bcb6f1f1b10188ad2a9b65f71e7cd7cf4590b38de4e80927`.

Tested `src/main.rs` SHA-256:
`df003504c4cd0fb56d4914fa044589d39b96ceccadb4cbae75f26cb73170d0de`.

## Interpretation

The candidate indexes only terms present in the queries and retains positions
only for phrase terms. Boolean queries use sorted document-ID lists; phrase
queries start from the shortest positional list and check exact offsets in the
others. The tokenizer still counts unindexed tokens, so dropping unused terms
does not alter adjacency. No ablation was run: the measurements cover the
language change, compact lists, selective indexing and query algorithms together.
Codex implemented the candidate and ran the checks.

## Same-source observation with the C bitset push

Commit `17838c7` changed C; this track's source was unchanged. Run
`run-856be110b4b8` passed all five gates and recorded 30.56437
machine points. The board retained the prior best 30.832131.
At 21:04:13 UTC, the overall display was 44.759, rank 38.

## Document shards and bounded summaries — 2026-10-30

A private phase-timing run of the accepted single-threaded code spent
3.576519 s building the index and 0.662760 s on queries and output. The new
implementation divides the input into up to six contiguous document ranges.
Each worker builds an independent index and answers all queries for its range.
It returns each query's count, ID sum and ten smallest matching IDs. Combining
those summaries preserves Boolean set operations and document-local phrases;
no full index merge or in-memory corpus copy is needed.

The first prototype was correct and faster, but failed its memory gate. Three
full-data pairs had median times of 4.19 s for the accepted code and 1.31 s for
the prototype, with peak process RSS of 369,928 and 847,928 KiB. A private
capacity probe found 444,833,280 bytes of posting and term-vector capacity,
plus 496,486,540 bytes in result summaries. Although each summary had at most
ten IDs, collecting `ids.into_iter().take(10)` reused the allocation of the
whole answer vector. The summary therefore retained far more capacity than
its length suggested. Copying the first ten IDs from a borrowed slice into a
fresh vector fixed that retention. No allocator setting was changed.

The corrected candidate was compared with the accepted single-threaded Rust
binary on the same otherwise idle Intel Core i5-12500H/Ubuntu 24.04 WSL2 machine.
Both used Rust 1.99.0 release optimization, LTO, one codegen unit and panic-abort.
Three pairs were interleaved baseline/candidate, candidate/baseline,
baseline/candidate. Each run read inputs, built indexes, answered queries and
wrote the complete output. The table gives median wall time and the maximum
process RSS among three runs, not official T or cgroup memory.peak M.

| Public seed 1 | Accepted wall time | Six-worker wall time | Accepted peak RSS (KiB) | Six-worker peak RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 0.18 s | 0.06 s | 30,200 | 39,348 |
| Full | 4.16 s | 1.21 s | 369,928 | 437,364 |

Full timings were 4.16, 4.20 and 4.16 s for the accepted binary, and 1.19, 1.24
and 1.21 s for the candidate: a 3.44× median ratio with an 18.2% RSS increase.
Sample RSS increased 30.3%. Parallel indexes and per-worker query summaries
still have a memory cost. A simple nonblank, noncomment source-line count grew
from 166 to 238; the board's normalized S may differ. No dependency was added.

Correctness checks passed:

- Every possible byte split in a fixture with UTF-8, empty documents and
  multi-digit IDs produced the same counts, sums and first ten IDs. A separate
  1,000-match case checked that a returned ten-ID summary retained no more than
  ten elements of vector capacity.
- The host worked example, all six host edge-case methods, 64-bit sums,
  repeated phrases, unindexed-token positions, a long document crossing several
  ranges and an empty corpus matched the host reference and an independent
  per-document predicate oracle. Worker settings 1, 2, 6 and 12 were checked;
  the last is clamped to six.
- Tiny seeds 28 and 29 passed before reserved seed 30 was generated. Each passed
  at one, two and six workers, included every host-listed edge category, and
  matched both independent implementations. Source bytes stayed frozen.
- All timed sample/full outputs matched the published hashes. The offline-built
  image also produced the full expected output as uid 65534, with a read-only
  root, no network, six CPUs, 8 GB memory without swap and a 1 GB tmpfs. Its size
  was 30,047,563 bytes. This was not the official scoring harness.

Tested `src/main.rs` SHA-256:
`1c09f4dd9e5928b9b182be994eff3edd281fe09bfee768a7c1f6e58f213f1696`.

Codex implemented and validated this change. Official live-board run
`run-a67cb4dd29a3`, commit `57f452c`, finished at 21:35:15 UTC with all five
gates passing. It recorded T = 1.1123 s, M = 460,935,168 bytes, S = 246,
D = 1, K = 8.381392 and 46.165807 machine points. This exceeds the prior B
best of 30.832131 by 15.333676 points, calculated from the two official values.
B now contributes to the team's best three tracks with A and D.

At 21:36:09 UTC, the board displayed overall 47.174 and rank 36. These are
live machine results, not the final judged total or an award.


## Streaming phrase matching — 2026-10-30

A private diagnostic of the accepted six-worker implementation measured
0.925–1.025 s building each shard's index and 0.160–0.174 s answering its queries.
Across the six indexes, position-vector capacity occupied 289,859,008 bytes of
444,833,280 bytes in posting and term metadata allocations. These are capacity
counts, not complete process memory measurements; simultaneous per-worker times
must not be added to obtain wall time.

The new implementation builds a token-ID Aho–Corasick automaton from the actual
phrase queries. Workers find phrase matches while reading documents, retaining
matching document IDs instead of every term occurrence and position. Boolean
queries keep their sorted-list operations. Phrase-only terms no longer need
Boolean posting lists. The six document shards and bounded result summaries
are unchanged. No external crate or ready-made search engine was added.

The accepted and candidate binaries used Rust 1.99.0 with release optimization,
LTO, one codegen unit and panic-abort on the same Intel Core i5-12500H/Ubuntu
24.04 WSL2 machine. Three pairs were interleaved baseline/candidate,
candidate/baseline, baseline/candidate. Each run included query parsing,
automaton or index construction, document reading, query evaluation and output.

| Public seed 1 | Accepted median wall | Candidate median wall | Accepted max RSS (KiB) | Candidate max RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 0.05 s | 0.05 s | 41,100 | 21,076 |
| Full | 1.22 s | 1.13 s | 437,568 | 185,296 |

Full timings were 1.19, 1.22 and 1.22 s for the baseline, and 1.15, 1.13 and
1.12 s for the candidate. Median wall time fell 7.4%, and maximum process RSS
fell 57.7%. GNU time's process RSS is not official cgroup memory.peak. A simple
nonblank, noncomment source-line count grew from 238 to 296; the board's
normalized S may differ. The measurements cover this implementation as a whole,
including removal of phrase-only Boolean postings; no ablation isolates each part.

Correctness checks included every byte boundary in the shard fixture, bounded
summary capacity, the host worked example and all six host edge-case methods.
An independent per-document predicate oracle also checked all binary token
sequences through length eight against overlapping and suffix-related phrases,
including repeated patterns, duplicate queries, unknown-token resets and
separate documents. Existing 64-bit sums, long-document and empty-corpus checks
passed at worker settings 1, 2, 6 and 12, with 12 clamped to six.

Tiny seeds 31 and 32 passed before reserved seed 33 was generated, with frozen
source and exact host/reference/oracle agreement at one, two and six workers.
Every timed sample/full output matched its published hash. The offline-built
30,054,392-byte image produced the exact full output as uid 65534, with a
read-only root, no network, six CPUs, 8 GB memory without swap and a 1 GB tmpfs.
This was not the official scoring harness.

Tested `src/main.rs` SHA-256:
`93b3c83be530f5b756710a185252290821655186148769cb209375b0086a2ea8`.

Codex implemented and validated this change. Its official result is pending
at this log entry; the local measurements do not establish a final score or award.
