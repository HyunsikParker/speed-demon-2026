# Track B: performance log

## Live-board history

The first candidate replaces the Python index with a standard-library Rust
query-term index.

| Run | Commit | Result | T (s) | M (bytes) | K | Machine points |
|---|---|---|---:|---:|---:|---:|
| `run-750270f3b3c2` | `bc07ea2` | Correct | 5.433148 | 387,010,560 | 4.136591 | 30.832131 |
| `run-27458ab5e1fd` | `82216f0` | Correct, unchanged B source | 5.915574 | 389,976,064 | 3.958284 | 29.875346 |

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
