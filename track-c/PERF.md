# Track C: performance log

## Live-board history

The first candidate replaces the Python CSV/object pipeline with a single-threaded
Rust parser, a direct-index region join and shared name IDs. No participant
Track C board measurement has been recorded yet. The run ID and measurements
will be added after the board returns them.

## Local validation — 2026-10-30

Machine: Intel Core i5-12500H, Ubuntu 24.04 under WSL2, Rust 1.99.0 and Python 3.12.
Both programs ran on the same otherwise idle development machine. Each timed
program read both inputs, performed the join and aggregation, and wrote the
complete output. GNU `time` recorded one wall-time and peak-process-RSS
observation per program and scale. These are not official five-run T or cgroup
`memory.peak` M values.

| Public seed 1 data | Python reference wall time | Rust wall time | Reference peak RSS (KiB) | Rust peak RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 0.88 s | 0.09 s | 75,032 | 17,204 |
| Full | 8.19 s | 0.96 s | 362,500 | 42,064 |

The full-data pair showed a wall-time speedup of about 8.53×. One pair does not
establish repeatability or predict the board coefficient. Local reference
Python 3.12 also differs from official Python 3.13.

Correctness checks passed:

- The official worked example and wider-grammar fixture, including literal BOM
  bytes inside text, no initial BOM, and final-record EOF, LF and CRLF variants.
- Header-only records and 7,000 records with 64-bit totals, quoted CR/LF/BOM bytes,
  escaped quotes, trailing-space names, unknown regions and negative cents.
- All these cases matched the CSV reference and an independent regex-based byte
  parser/aggregator; fixtures with literal expected output matched that too.
- Tiny seeds 7 and 8, then reserved seed 9 generated only after prior checks
  passed. Each contained every host-listed edge-case category and matched both
  independent implementations. The candidate source remained unchanged.
- Published sample/full seed 1 input hashes and exact output hashes.
- Full output from the offline-built image as uid 65534, with no network,
  read-only root, an 8 GB memory limit, no swap and a 1 GB tmpfs. The image was
  30,041,127 bytes. This check did not use the official scoring harness.

Full output SHA-256:
`d8088b1cd42ba1338b1345b107b55db7b96ef1f5066f3ff64fd26a8912a0f536`.

Tested `src/main.rs` SHA-256:
`b22136e1f71cd229e56a2d0eebfeee9babb7433f15a2f0dbc6ad7957a3e7f996`.

## Interpretation

The reference decodes CSV values into Python strings, validates numeric fields
with regular expressions, and stores names separately in each country's set.
This candidate scans buffered bytes, counts notes without retaining them, and
stores one copy of each observed name plus integer IDs in country sets. It also
replaces per-record region hash lookup with a direct index. No ablation was run;
the measurements cover these changes together. No official score, rank or award
is claimed here. Codex implemented the candidate and ran the checks.
