# Track B: performance log

## Live-board history

This first candidate replaces the Python index with a standard-library Rust
query-term index. It has passed the local checks below; an official live-board
run has not yet been observed. No official score is claimed here.

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
