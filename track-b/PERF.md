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

Codex implemented and validated this change. The official result is pending
at this log entry; these local measurements do not establish a final score
or an award.
