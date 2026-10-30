# Track C: performance log

## Live-board history

The first candidate replaces the Python CSV/object pipeline with a single-threaded
Rust parser, a direct-index region join and shared name IDs.

| Run | Commit | Result | T (s) | M (bytes) | K | Machine points |
|---|---|---|---:|---:|---:|---:|
| `run-a386410cf9fb` | `0a3e541` | Correct | 1.498349 | 41,844,736 | 4.471266 | 32.521528 |
| `run-1aa8da2699b7` | `bc07ea2` | Correct, unchanged C source | 1.378084 | 41,967,616 | 4.659556 | 33.417225 |
| `run-003ee484731c` | `82216f0` | Correct, unchanged C source | 1.384427 | 41,922,560 | 4.649868 | 33.372031 |
| `run-b8ed3a89e330` | `17838c7` | Correct, dynamic country bitsets | 1.091973 | 18,083,840 | 6.147218 | 39.433931 |

The [official live board](https://speeddemon.dev/board/) reported this run on
2026-10-30, from 20:21:22 to 20:21:25 UTC. Check, build, correctness, timeout and
sandbox gates passed. The board reported S = 172 and D = 1. At 20:22:29 UTC, the
team's three scored tracks yielded an overall 40.591, rank 40. B was still
unscored. The overall board uses the geometric mean of the best three tracks;
these are live machine points, not final judging or an award.

The next run accompanied the first Track B implementation push, with unchanged
C source. It ran from 20:38:00 to 20:38:02 UTC; all five gates passed
and S = 172, D = 1 were unchanged. At 20:39:16 UTC, the team's overall display was
40.969, rank 40. This same-source measurement is not an algorithmic improvement.

The next run accompanied D's bidirectional-search update with unchanged C
source. All five gates passed; S = 172 and D = 1 were unchanged. Its latest
33.372031 points were below the prior best 33.417225, which the page
retained. At 20:51:13 UTC the overall display was 42.042, rank 38. This is a
same-source control, not an algorithmic regression or improvement claim.

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
the measurements cover these changes together. The official live-board result
is recorded above; no final score or award is claimed. Codex implemented the
candidate and ran the checks.

## Dynamic country bitsets — 2026-10-30

A phase-timing copy of the accepted Rust implementation spent 1.165569 s before
output and 1.165805 s in total on full seed 1. That observation covers reading,
parsing, the join and distinct-name aggregation together; it does not isolate
one lookup's cost. This candidate changes only the per-country distinct-name
structure: dynamically growing bitsets replace hash sets of global name IDs.
The global byte-string interner, CSV parser, region join and output stay the same.

The comparison used the same machine and compiler as above. Both Rust binaries
used release optimization, LTO, one codegen unit and panic-abort. Three runs of
each implementation were interleaved baseline/candidate, candidate/baseline,
baseline/candidate. Every run read the inputs and wrote the complete output.
The table reports median wall time and the maximum process RSS among three runs.

| Public seed 1 | Accepted Rust wall time | Bitset wall time | Accepted peak RSS (KiB) | Bitset peak RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 0.09 s | 0.09 s | 17,164 | 18,144 |
| Full | 0.93 s | 0.69 s | 42,060 | 18,548 |

Full timings were 1.04, 0.93 and 0.92 s for the accepted binary, and 0.68, 0.69
and 0.69 s for the bitset candidate. The median ratio was 1.35×; peak RSS fell
55.9%. Sample memory increased 5.7%, and its median timing was unchanged at the
measurement's 0.01 s precision. A bitset reserves space up to the largest global
ID seen in a country, so it can cost more for sparse memberships. These are
local observations, not official T, M or K values.

All prior grammar, header-only and wide-sum checks passed against the host CSV
reference and the independent regex-based byte oracle. An added fixture used
70,000 distinct names, repeated IDs around 63/64/65, 127/128 and 65,535/65,536,
and the same names across several countries, including a late sparse country
and unmatched records. Empty, trailing-space, quoted, BOM and CR/LF-containing
names stayed distinct exactly as required. Tiny seeds 16 and 17 passed before
reserved seed 18 was generated; source bytes stayed frozen throughout.
Published sample/full input and output hashes matched on every timed run.

The offline image produced the full expected hash as uid 65534 with a read-only
root, no network, six CPUs, 8 GB memory without swap and a 1 GB tmpfs. Image size
was 30,040,164 bytes. The local harness check is not the official scoring run.
The source grew from 164 to 173 nonblank, noncomment lines under a simple local
count; the board's normalized S may differ. No library dependency was added.

Candidate `src/main.rs` SHA-256:
`15dea14b9a159bf0360ae55e4fcb056892a31f2b2dbc13f7618bfcfff54be1c9`.

Codex implemented and validated this change. The official live-board run
`run-b8ed3a89e330` for commit `17838c7` finished at 21:03:53 UTC with all five
gates passing: T = 1.091973 s, M = 18,083,840 bytes, S = 181, D = 1,
K = 6.147218 and 39.433931 machine points. This exceeds the prior best
33.417225 by 6.016706 points (calculated from the two official values).
At 21:04:13 UTC, the board displayed overall 44.759, rank 38. D also recorded
a better same-source rerun; the overall change cannot be attributed solely to
C. These are live machine results, not final judging or an award.

## Rejected name-hash experiment — 2026-10-30

Sparse instrumentation sampled every 257th record at three offsets. The
extrapolated text-decoding times were 0.187–0.222 s; name-interner times were
0.350–0.384 s. These estimates include timer overhead and sampling effects, so
they identify a candidate region rather than an exact CPU-time breakdown.

An independent Rust implementation of 64-bit FNV-1a, following
[RFC 9923](https://www.rfc-editor.org/rfc/rfc9923.html#section-2), replaced only
the global map's hasher. The map still compared complete name bytes. All
correctness checks passed, including a private build that returned zero for
every hash: distinct names, empty values, quotes, BOM bytes and trailing spaces
still remained correct. Tiny seeds 19 and 20, then reserved 21, and the offline
full-data container also matched the expected outputs.

Five interleaved full-data runs had a median of 0.69 s for both implementations.
The accepted times were 0.66, 0.68, 0.69, 0.70 and 0.71 s; the candidate times
were 0.67, 0.67, 0.69, 0.69 and 0.69 s. Maximum process RSS was 18,532 versus
18,528 KiB. The additional hashing code brought no measured median improvement,
so the candidate was rejected locally and never submitted. The accepted
standard-library hasher was restored. This result does not establish that
hashing is free; it rules out this replacement under the measured conditions.

## Rejected field-decoder experiment — 2026-10-30

A separate candidate copied unquoted fields as contiguous slices and decoded
quoted fields one segment at a time, between quote delimiters. The standard
name hasher and country bitsets stayed unchanged. It passed the existing
oracles, 338 additional name/note combinations under minimal and explicit
text quoting, tiny seeds 22 and 23 followed by reserved 24, and the offline
full-data container. The added cases included empty values, commas, escaped
quotes, CR/LF, BOM bytes and 4,096-byte fields.

Five interleaved full runs again had a median of 0.69 s for both binaries.
The baseline times were 0.77, 0.69, 0.68, 0.80 and 0.69 s; candidate times were
0.72, 0.69, 0.69, 0.72 and 0.69 s. Maximum process RSS was 18,548 KiB for both.
The timing gain did not meet the registered gate, so the change was kept out
of the submission and the accepted decoder was restored. These two negative
experiments leave the previously scored bitset implementation intact.

## Same-source observation with the B shard push

Commit `57f452c` changed B; this track's source was unchanged. Run
`run-8ca90f47ddae` passed all five gates and recorded 39.509831
machine points. This is a new same-source best, up from 39.433931;
it does not establish a new C algorithm improvement. C is now outside
the best three tracks used for the displayed overall score.
At 21:36:09 UTC, the overall display was 47.174, rank 36.

## Same-source observation with the B automaton push

Commit `a3a4156` changed B; this track's source was unchanged. Run
`run-cf74054c789c` passed all five gates and recorded 39.615362
machine points. This is a new same-source best, up from 39.509831;
it does not establish a new C algorithm improvement. C remains outside
the best three tracks used for the overall display.
At 21:50:14 UTC, the overall display was 48.330, rank 35.

## Same-source observation with the D radix push

Commit `47a7c57` changed D; this track's source was unchanged. Run
`run-d4e568e622b5` passed all five gates and recorded 38.602265
machine points. The board retained the prior best 39.615362. The slower
measurement came from unchanged source and is not evidence of a code change.
At 22:11:03 UTC, the overall display was 48.815, rank 34.

## Same-source observation with the A parallel-range push

Commit `014bdb7` changed A; this track's source was unchanged. Run
`run-4e3efdc75f8e` passed all five gates and recorded 39.022841
machine points. The board retained the prior best 39.615362.
At 22:23:46 UTC on 2026-10-30, the overall display was 51.600, rank 35.
These are live board results, not final judging or an award.
