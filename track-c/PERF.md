# Track C: performance and validation

Snapshot: 2026-10-31. The current source combines inline membership with
read-only mapping and parallel quote-parity boundary discovery. It passed the
local adoption gate below; its first official board run is pending. The recorded
official result still belongs to the prior inline-membership pipeline.

## Recorded official result

Run `run-52f0819fc621`, on commit `1c06a7cd760f25d771fb196b6e111e9b74d2666c`,
finished at 05:55:45 UTC with all five gates passing and **55.455380 machine points**.

| Official axis | Value |
|---|---:|
| T: process time | 0.187834 s |
| M: cgroup memory peak | 20,283,392 bytes |
| S: source size | 401 |
| D: dependency axis | 1 |
| K: coefficient | 12.856051 |

This improved the prior best of 51.737524 by 3.717856 points and exceeded the
registered 53.4-point target. The best remained unchanged in the 08:02 UTC
readback. [MEASUREMENTS.json](../MEASUREMENTS.json) records the own-track observations;
no final judge score or award is asserted.

## Accepted design

The [implementation notes](README.md) describe the CSV grammar and exact join.
The input is mapped read-only. Six byte chunks count quotes in parallel using
runtime-guarded AVX2 masks or a scalar fallback. Prefix parity establishes the
quote state at five raw cuts; short forward scans move them to complete-record
boundaries. Six workers borrow the disjoint ranges, forming exact logical-record
batches targeting 64 KiB. Quoted newlines and oversized records remain intact.
This removes producer-owned input buffers and queues.

Six workers decode names, count decoded note bytes and accumulate numeric totals.
Region IDs index a compact country table; absent regions map to `_UNMATCHED`.
Money remains signed integer cents throughout parsing and aggregation.

Distinct-name counting is shared across workers. A batch groups requests by one
of 64 mutex-protected shards, locking each nonempty shard once. The first exact
`(name, country)` pair sets its membership bit and increments one worker's count
under that lock. The final sum therefore counts each pair once.

Names of at most 16 bytes use complete inline keys and four country-bitset words
when the table has at most 256 country groups, including `_UNMATCHED`. Length
is part of identity. Longer
names and larger country tables retain the full-byte map and variable-size
bitsets. Worker buffers and request-vector capacities are reused across batches.

## Local evidence for the accepted stages

The first two stages passed their separate whole-program adoption gates. Their baselines
differ, so the ratios below are not independent effects to multiply together.

| Stage | Baseline → candidate image time | Baseline → candidate native peak RSS | K-ratio proxy |
|---|---:|---:|---:|
| Parallel pipeline with retained worker scratch, versus serial source | 0.710919 → 0.183349 s | 18,548 → 24,000 KiB | 1.677626 |
| Inline membership, versus that accepted pipeline | 0.186579 → 0.152485 s | 24,060 → 19,828 KiB | 1.133347 |

Each row used five interleaved pinned-image pairs and the published child timer
on an Intel Core i5-12500H running Ubuntu 24.04 under WSL2. Times are middle-three
means and include input reading, framing, parsing, membership, aggregation and
output. Both accepted batches passed their spread checks. Native peak process
RSS and nonblank source-line counts are local proxies for the board's M and S.

The pipeline's first timing batch failed its baseline stability check and was
not used for adoption. One registered operational repair retained the exact
sources and images, waited for a quiet foreground WSL session, and ran fixed
warm-ups before the original five-pair measurement. It changed no service,
algorithm, parameter or acceptance threshold. The unstable batch remains in the
history; its apparent gain was not treated as evidence.

## Correctness checks

The accepted inline-membership change passed seven Rust tests and 15 correctness
groups. They cover every inline-key byte position and length, zero bytes, the
16/17-byte boundary, country-word boundaries through 513 countries, concurrent
first-pair updates and scratch reuse. Tiny seeds 43 and 44 passed before reserved
seed 45 was generated with the source frozen.

CSV validation covers escaped quotes, embedded CR/LF/BOM bytes, trailing spaces,
empty fields, unknown regions, negative cents, and EOF/LF/CRLF record endings.
Vector and scalar record scanners are compared across quote states and split
positions. Host-reference and independent byte-parser results are checked.
The published full seed-1 `aggregates.txt` SHA-256 is:

`d8088b1cd42ba1338b1345b107b55db7b96ef1f5066f3ff64fd26a8912a0f536`

Build and exact-output checks are described in the [repository README](../README.md#build-and-check).

## Rejected alternatives and limits

The later fused-fingerprint membership table passed ten Rust tests and 15 check
groups, including deliberate full-fingerprint collisions, table growth and
concurrent updates. Pinned-image time fell from 0.145918 to 0.143155 s, a
1.019303× ratio, below the fixed 1.18× requirement. Its source-line proxy grew
from 393 to 457; the resulting K proxy was 0.985945, below 1.05. Its RSS ratio
passed the 1.15 limit. The candidate was not submitted and the accepted source
was restored.

Other measured trials included vector field searches, worker-side framing,
borrowed fields and shared input blocks. Their exact outcomes remain in the
history. A reduction in copying, allocations or instructions was not accepted
as a speed improvement without a passing whole-program comparison.

Producer throughput, decoding and shared membership still contribute work.
Their sampled CPU locations are not elapsed-time shares. Long-name and
large-country-table inputs use a different storage path, so measurements on the
published input do not characterize every legal workload.

## Whole-record structural parsing: rejected local trial

A later candidate scanned each complete record for outside-quote commas and
cumulative quote counts. One AVX2 pass could cross field boundaries; scalar
code handled tails and unsupported CPUs. Quote counts gave the decoded note
length without decoding the note byte by byte. Unescaped names borrowed their
input slice; escaped names used the worker's scratch buffer. Producer framing,
membership storage, six workers, 64 shards and batching stayed unchanged.

Ten Rust tests and 15 check groups passed. The new tests compared scalar and
vector field metadata at every 32-byte boundary and unaligned offset, and
checked empty fields, escaped quotes, Unicode, BOM bytes and a 2 MiB note.
Fresh tiny seeds 124 and 125 passed before reserved seed 126. Host-reference
and independent CSV outputs agreed, as did the published sample/full hashes
and all 14 image-run outputs.

Five interleaved pinned-image pairs gave middle-three means of 0.150447 s for
the accepted source and 0.147206 s for the candidate: a 1.022018 speed ratio.
Native peak process RSS was 20,060 versus 20,016 KiB, a ratio of 0.997807.
The nonblank source-line proxy rose from 393 to 458, giving a K-ratio proxy of
0.988437. Both timing spreads passed. Speed missed the fixed 1.18 requirement
and K missed 1.05; RSS passed its 1.15 limit.

The candidate was not submitted. The accepted source and release binary were
restored, and their full-output hash was checked again. These local results
do not establish an official score change. Replacing per-field scans with
shared metadata did not meet the whole-program adoption gate in this trial.

## Memory-mapped ranges: rejected local trial

This candidate mapped `records.csv` read-only and split it into six ranges at
complete CSV record boundaries. A sequential quote-parity scan found the cuts;
each worker then framed and parsed its borrowed range. Input buffers and
producer channels were removed. Field decoding, country lookup and exact
shared-name membership stayed unchanged.

Eight Rust tests and 17 check groups passed, including complete-record cuts,
quoted newlines, an oversized record, concurrent membership, and fresh tiny
seeds 134 and 135 followed by reserved seed 136. Both image-built binaries
matched the published sample/full outputs. All 14 timed image outputs matched.

Five interleaved pairs after two warm-up pairs gave middle-three means of
0.150218 s for the accepted source and 0.154878 s for the candidate. The speed
ratio was 0.969912; both spread checks passed. Peak container-cgroup memory
fell from 24,346,624 to 18,317,312 bytes, a ratio of 0.752355.

The data was warmed on the host before each container existed. A common trusted
shell kept each Docker cgroup alive after the published child timer finished,
so its `memory.peak` could be read. These are local WSL container measurements;
the holder and writable timing mount differ from the official parent-cgroup
harness and its isolation. Process RSS was not used for this mapping trial.

The source-line proxy was 409 versus 412, including the unchanged build files.
Both dependency weights remained 1. The corrected K-ratio proxy was 1.041371;
an erroneous image-size factor in the initial calculation was removed during
verification, with the original record retained. The decision was unchanged:
speed missed 1.18 and K missed 1.05, while memory passed its 1.05 ratio limit.
The candidate was not submitted, and the accepted source was left unchanged.

## Parallel quote-parity boundaries: local adoption

The first mapped-range candidate still enumerated every record ending in a
sequential pass to choose five cuts. This candidate replaces that pass with six
quote-only parity scans and five boundary searches. CSV decoding, six parsing
workers and the 64 membership shards are unchanged.

A fixed batch used two warm-up pairs and five interleaved baseline/candidate
pairs on the same Victus machine. The pinned-image child timer measured these
whole-program results:

| Measure | Prior pipeline | Parallel-parity mapping |
|---|---:|---:|
| Middle-three mean process time | 0.149709 s | 0.124672 s |
| Maximum measured container memory peak | 20,582,400 bytes | 18,612,224 bytes |
| Nonblank source-line proxy, including Dockerfile and manifest | 409 | 491 |
| Dependency axis | 1 | 1 |

Time fell by 16.72% (1.200820× speedup) and measured memory by 9.57%.
The K-ratio proxy was 1.087865. These passed the fixed requirements of at least
1.18× speedup, memory ratio at most 1.05 and K proxy at least 1.05; both timing
spread checks passed. One baseline sample took 0.202381 s. The preregistered
middle-three calculation drops each arm's fastest and slowest samples, and the
raw timings are retained.

Eleven Rust tests and 17 validation groups passed. The added tests compare
scalar/vector quote parity at every alignment and tail, check every possible
cut in a quoted-record fixture, and exercise a long record that spans multiple
partitions. Fresh tiny seeds 137 and 138 passed before reserved seed 139; source
bytes remained frozen. Host-reference and independent byte-parser outputs,
published sample/full outputs, and all 14 image outputs matched exactly.

Memory comes from each Docker cgroup's `memory.peak`, with the input prewarmed
outside the container and a common trusted holder keeping the cgroup readable
after the timed child exits. This is a local measurement, without the official
parent harness or its timing-file isolation. Source size is a nonblank-line
proxy, not official `scc`. D is one plus direct runtime dependencies, so both
arms have D = 1; image size is only a build-limit check. These results qualify
one official trial and do not establish a board score or prize.

## Record and assistance

The [complete experiment log below](#complete-experiment-log) retains the earlier log byte-for-byte, including
rejected trials and the operational timing repair. Historical “current” and
“pending” statements refer to their original snapshots.

Codex developed the implementation and validation.

## Complete experiment log

<details>
<summary>Open the full chronological record, including rejected trials and measurement repairs</summary>

# Track C: performance log

## Fused inline membership — rejected local candidate, 2026-10-31

This candidate computed one fingerprint for each short decoded name and reused
it for shard selection and an open-addressed membership table. Each table slot
kept the complete two-word key, its byte length and four country bitsets.
Full-key equality decided identity; equal fingerprints did not merge names.
Long names and sets of more than 256 countries retained the existing fallback.
CSV parsing, framing, six workers, 64 shards and batch sizes stayed unchanged.

Ten Rust tests passed, including deliberately identical 64-bit fingerprints
for different keys, repeated table growth, an independent pair-set oracle,
empty names, all byte positions, country-word boundaries and concurrent
contention. The 15 recorded check groups covered those tests, the independent
CSV cases and fresh seeds 121–123, with 123 reserved. Published sample/full
outputs and all 14 image-run outputs matched.

In the fixed five-pair comparison, pinned-image whole-process time fell from
0.145918 to 0.143155 s, a speed ratio of 1.019303. Maximum process RSS was
19,888 versus 19,996 KiB. The nonblank source-line proxy grew from 393 to 457,
giving a source-adjusted K proxy of 0.985945. Both timing spreads passed.
The measured speed ratio missed the required 1.18, and K missed 1.05; the
RSS ratio of 1.005430 passed its 1.15 limit. These are local measurements and
proxies, not official scores or cgroup-memory results.

The small observed speed gain did not meet the efficiency gate. The candidate
was not submitted, and the accepted source and native executable were restored.

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

## Same-source observation with the A lazy-group push

Commit `86a95c8` changed A; this track's source was unchanged. Run
`run-48120d79e8cf` passed all five gates and recorded 36.7281
machine points. The board retained the earlier best 39.615362.
At 22:38:49 UTC on 2026-10-30, the overall display was 51.765, rank 35.
This rerun is not evidence of a new algorithm, final judging or an award.

## Same-source observation with the D compact-storage push

Commit `e0378c5` changed D; this track's source was unchanged. Run
`run-9fdffd9293cb` passed all five gates and recorded 39.29661
machine points. The board retained the earlier best 39.615362.
At 22:50:46 UTC on 2026-10-30, the overall display was 51.884, rank 35.
This rerun does not establish a new algorithm improvement, final judging or an award.

## Same-source observation with the D CSR-index push

Commit `2f5cdaf` changed D; this track's source was unchanged. Run
`run-86139960c3f2` passed all five gates and recorded 38.034961
machine points. The board retained the earlier best 39.615362.
At 23:07:03 UTC on 2026-10-30, the overall display was 52.170, rank 34.
This rerun does not establish a new algorithm improvement, final judging or an award.

## Same-source observation with the popped-vertex meeting push

Commit `b94f7ff` changed D; this track's source was unchanged. Run
`run-7e5e1cff1ef3` passed all five gates and recorded 38.945851
machine points. The board retained the earlier best 39.615362.
At 23:53:09 UTC on 2026-10-30, the overall display was 52.240, rank 35.
This rerun does not establish a new algorithm improvement, final judging or an award.


## Rejected local experiment: bounded parallel parsing

On 2026-10-31, a six-worker candidate consumed CSV batches targeting 1 MiB
and ending only at logical record boundaries. Oversized records stayed intact.
Each worker kept exact decoded-name/country sets; the final merge took their
union before counting distinct names. All 14 correctness
checks passed, including a record larger than a batch, cross-worker duplicates,
BOMs, an unterminated final line, seeds 25/26 and the frozen seed-27 holdout.
The published sample/full hashes and the offline, unprivileged container passed.

Five interleaved full-size pairs measured median wall time of 0.71 s for the
accepted serial version and 0.37 s for the candidate. Maximum process RSS rose
from 18,548 to 118,620 KiB. The local coefficient-ratio proxy was
0.906063, below the fixed 1.60 gate. It uses GNU time process RSS and
nonblank, noncomment source lines, not the official cgroup memory or scc count.
A roughly 1.59 coefficient gain would be needed just to reach the lowest of
the current best three track scores. No official score improvement is claimed.

The candidate was not pushed, and the accepted serial source was restored.
The first validation attempt stopped at Python's default CSV field-size limit
in the added 2 MiB-field test. Raising that test-process limit left the candidate
and reference algorithm unchanged; the rerun then completed. Both attempts
are recorded separately.

## Shared membership and batched locking — 2026-10-31

A capacity diagnostic of the rejected six-map version found 598,023 name
entries across workers for 100,520 globally distinct names. The tracked peak
capacity of sent, blocked and consuming batches was 23,625,728 bytes. These
counts describe duplicated storage and buffer capacity; worker snapshots were
taken at different times, and the buffer counter excludes the unfinished
producer buffer and allocator overhead.

The next candidate, v7, kept one exact name-to-country-bitset map behind 64
locks. Each first `(name, country)` pair incremented one worker's counter
while holding its shard lock. Six parsing workers consumed logical-record
batches targeting 64 KiB; oversized records remained intact. This removed the
per-worker name maps and their final union merge.

Two process-scoped `task-clock:u` profiles at 999 Hz collected 1,225 and 1,285
CPU samples. Exact-binary source resolution attributed 39.27% and 36.96% to
membership updates. Across all samples, mutex leaf functions accounted for
14.37% and 14.16%; their attribution is not wholly nested under membership.
Another 226 and 266 samples remained unresolved or belonged to other DSOs.
These are CPU sample locations, not wall-time shares or measured speedup
opportunities. The first offline decoding attempt exceeded its 10-second
limit; decoding the same saved samples completed with a 60-second limit.
The workload was not rerun for that repair.

Candidate v9 grouped each batch's decoded-name requests by the same 64 shards,
then acquired each nonempty shard once. It retained six workers, 64 KiB
batches and full-byte name equality. Batch arenas added temporary copies.

Each candidate passed the existing CSV oracles, concurrent duplicate-count
checks, published sample/full hashes and three new tiny seeds. V9 also checked
duplicates within and across concurrent batches on one forced shard. The
reserved seeds were 30 for v7 and 33 for v9, with source frozen before each.

| Candidate | Accepted image time (s) | Candidate image time (s) | Accepted / candidate peak RSS (KiB) | Source-line proxy | K-ratio proxy |
|---|---:|---:|---:|---:|---:|
| Shared map, v7 | 0.718178 | 0.258539 | 18,540 / 24,432 | 173 / 268 | 1.476964 |
| Batched locks, v9 | 0.712441 | 0.254487 | 18,548 / 24,976 | 173 / 290 | 1.458962 |

Each row used five interleaved pairs on Victus WSL. Image times are the mean of
the middle three runs recorded by the published child timer; both spread
checks passed. RSS is the maximum from five native runs, and source size counts
nonblank, noncomment lines. The proxy combines these local measurements; it
is not the official cgroup-memory/scc coefficient or a board score. The two
candidates ran in separate batches, so their small timing difference does not
establish that batching improved on v7.

Both missed the fixed 1.65 K-ratio gate and were kept out of the submission.
The accepted serial source was restored. A further change needs evidence about
producer framing, channel blocking and worker processing on the critical path;
CPU sample percentages alone do not identify it.

## Pipeline phases and block framing — 2026-10-31

Batch-boundary timers in private builds compared the v7 and v9 pipelines.
Across two full-data v9 runs, median worker processing occupied 133.5–134.9 ms,
while the producer took 218.6–235.6 ms. Workers spent a median 85.4–100.1 ms
inside receive calls. Producer time outside send calls was 184.9–200.3 ms.
For v7, median worker processing was 209.9–221.1 ms and the producer took
245.3–248.1 ms. All eight sample, control and instrumented runs matched the
published output hashes.

Worker intervals overlap. Send and receive calls include channel overhead,
scheduling and waiting; producer time outside sends includes reading, framing,
copying and bookkeeping. The counters therefore support investigating the
producer, but do not isolate I/O cost or prove a causal wall-time saving.

Candidate v11 scanned each buffered input block for quotes and newlines.
On AVX2-capable x86-64 CPUs, 32-byte masks and prefix XOR tracked quote parity
across vectors and input blocks. Other CPUs and the final short tail used the
scalar scanner. Copies into the existing 64 KiB logical-record batches occurred
at dispatch or input-block boundaries. The six workers and v9 membership
updates stayed unchanged.

The vector load is bounded by the remaining slice length and guarded by
runtime feature detection. Its unaligned-load and feature-detection contracts
are documented in Rust's [_mm256_loadu_si256](https://doc.rust-lang.org/core/arch/x86_64/fn._mm256_loadu_si256.html)
and [is_x86_feature_detected!](https://doc.rust-lang.org/std/macro.is_x86_feature_detected.html)
references. Private tests compared vector/scalar boundaries across quote states,
byte values, short tails and split positions. All four Rust tests, the existing
CSV/concurrency oracles, tiny seeds 34/35 followed by reserved 36, and the
published sample/full hashes passed.

Five interleaved pinned-image pairs measured 0.720779 s for the accepted serial
version and 0.192418 s for v11, using middle-three means; both spread checks
passed. The speed ratio was 3.745895. Maximum native process RSS was 18,548
versus 25,252 KiB, and the source-line proxy rose from 173 to 343.
The resulting local K-ratio proxy was 1.642067, just below the fixed 1.65 gate.
These remain development-machine proxies, not official board measurements.

The near miss was recorded without lowering the threshold or repeating the
same timing trial. V11 was not submitted; its source and measurements were
preserved, and the accepted serial source was restored. Before another change,
the remaining per-batch scratch allocations are a candidate for measurement,
not an established bottleneck or a promised gain.

## Worker-owned scratch buffers — 2026-10-31

A private diagnostic counted 197,120 nonzero-capacity request-vector lifetimes
across 3,080 full-input batches in v11. Their cumulative final capacities were
114,050,688 bytes. Another 9,240 scratch-buffer lifetimes accounted for
52,930,304 bytes of cumulative capacity. These are buffer-lifetime counts,
not allocator-call counts, peak RSS or an attribution of CPU time.

Candidate v13 moved the name buffer, note scratch buffer, decoded-name arena
and per-shard request vectors into each worker. It clears their lengths between
batches and retains their capacities. Framing, six workers, 64 shards and
64 KiB target batches stayed unchanged. Five private Rust tests and all 15
correctness groups passed, including long-to-short name reuse, empty batches,
concurrent duplicates, vector/scalar framing, CSV edge cases and tiny seeds
37/38 followed by the frozen seed-39 holdout. Published sample/full outputs
matched exactly.

The first timing batch failed its stability check: the accepted baseline's
middle-three spread was 0.087447 s, above the 0.05 s bound. Its apparent
K-ratio of 1.680064 was therefore insufficient for adoption. The run was
recorded as a measurement failure; it did not establish that the candidate's
mechanism failed.

A subsequent environment check found WSL services starting alongside a new
remote command. The original preflight had not excluded that startup activity.
Its contribution to the earlier timing spread remains unproven. One registered
operational repair kept a single foreground WSL session alive, waited for
uptime of at least 60 seconds and five consecutive quiet samples, and reused
the exact Docker images and source. Each phase then ran two fixed, unscored
warm-up pairs before the original five interleaved measurement pairs. No
service, power setting, source parameter or acceptance threshold changed.

The repaired batch measured middle-three image times of 0.710919 s for the
accepted serial version and 0.183349 s for v13, a ratio of 3.877416. Both
spread checks passed. Maximum native process RSS was 18,548 versus 24,000 KiB;
the source-line proxy was 173 versus 357. The resulting K-ratio proxy,
1.677626, passed the fixed 1.65 gate. All new full-input outputs matched.

These are local Victus WSL measurements using the published child timer,
native process RSS and nonblank, noncomment source lines. They are not the
official scoring machine, cgroup-memory measurement or scc count. The original
unstable batch remains recorded separately; the repair was one fixed batch,
not a selection among repeated attempts. Official evaluation is still needed
to establish this candidate's board score.

## Official live-board result for worker-owned scratch

Commit `b917cfa54cbf3e4cfaf3aebef9abd921bdc5c624` was pushed once on
2026-10-31 at 05:16:53 UTC. C run `run-78ca238aac1c` finished at 05:17:39 UTC
with all five gates passed: T 0.251930 s, M 24,592,384 bytes, S 365, D 1,
K 10.833043 and 51.737524 machine points. This improved C's prior best
39.649090 by 12.088434 points and placed C among the team's best three tracks.

At 05:18:33 UTC, the board displayed an overall score of 55.926 and rank 39.
A, B and D reran with unchanged sources and retained their earlier bests.
These are live-board observations, not final judging, an award or payment.

## Current pipeline phases and worker-side framing

Private timers on the accepted v13 source measured two full-input runs after
the startup-readiness check. Producer time was 175.2–182.3 ms: send calls took
83.9–89.3 ms, buffered reads 23.9–24.7 ms, record scanning 38.2–40.5 ms, and
remaining copying/bookkeeping 26.9–30.1 ms. Median worker time was
135.0–137.3 ms, including 67.8–68.0 ms of parsing and 66.0–68.0 ms of shared
membership updates. All six diagnostic outputs matched their published hashes.
Worker intervals overlap; call durations include scheduling and waiting, and
these instrumented timings do not isolate causal savings.

Candidate v15 made the producer find only the first complete record ending at
or after each batch target. Workers then found the internal record boundaries,
reusing their boundary vectors. The quote-parity kernel supported both modes.
Six workers, 64 shards, the 64 KiB target and 1 MiB read buffer stayed fixed.
Seven private Rust tests and all 15 correctness groups passed, including
minimum-boundary offsets, chunk carry, quoted newlines, oversized records and
tiny seeds 40/41 followed by reserved 42.

One fixed five-pair image batch measured middle-three means of 0.186995 s for
accepted v13 and 0.174319 s for v15. Both spread checks passed. The speed ratio
was 1.072719, the native peak-RSS ratio was 0.977900, and the source-line proxy
grew from 357 to 381. The incremental K-ratio proxy was 1.030257.

The candidate missed the preregistered speed ratio of 1.18 and K-ratio of 1.08
against the new accepted baseline. It was not submitted; v13 was restored.
The local reduction was measurable in this batch but insufficient for those
gates. The original serial-baseline gates and earlier negative results remain
unchanged. Further work needs a distinct mechanism supported by measurements
of the remaining parsing or membership cost.

## Current-source CPU samples and inline membership

Two private CPU profiles of accepted v13 collected 853 and 809 software
samples; the sample smoke, unprofiled full control and both profiled outputs
matched their expected hashes. Exact-binary inline locations attributed 171
and 181 samples to shared membership, 104 and 110 to text decoding, and 75
and 62 to integer parsing. These counts are not elapsed-time shares.

Offline analysis of the saved profiles matched libc's build ID to the local
library. Its resolved `memcmp`/`bcmp` entry branches to a short-byte comparison
block containing 146 and 134 samples. This identifies that block, not its
runtime callers. The analysis did not rerun or recapture the workload; other
unresolved samples remain unassigned.

Candidate v17 stores complete names of up to 16 bytes and four country-bitset
words directly in the sharded table when there are at most 256 countries.
The key includes the original byte length. Long names and larger country sets
keep the full-byte map and variable-size bitsets. Short requests no longer
copy names into the decoded arena. Workers, shards, batching and framing
remain unchanged.

Seven private Rust tests and all 15 correctness groups passed. They cover
all inline-key byte positions and lengths, zero bytes, the 16/17-byte name
boundary, country-word boundaries through 513 countries, concurrent first-pair
updates, scratch reuse and the wider CSV grammar. Tiny seeds 43/44 ran before
reserved seed 45; published sample and full outputs also matched.

After the startup-readiness check and fixed unscored warm-ups, one five-pair
image batch measured middle-three means of 0.186579 s for accepted v13 and
0.152485 s for v17, a speed ratio of 1.223591. Both spread checks passed.
Maximum native process RSS was 24,060 versus 19,828 KiB, a ratio of 0.824106.
The source-line proxy grew from 357 to 393. The resulting incremental K-ratio
proxy was 1.133347, passing the fixed speed, memory and efficiency gates.

These are local Victus WSL measurements using the published child timer,
native process RSS and nonblank, noncomment source lines. They do not establish
an official score or replace official cgroup-memory and scc measurements.
Official evaluation is still required for this candidate.

## Official live-board result for inline membership

Commit `1c06a7cd760f25d771fb196b6e111e9b74d2666c` was pushed once at
2026-10-31 05:55:08 UTC. C run `run-52f0819fc621` finished at 05:55:45 UTC
with all five gates passed: T 0.187834 s, M 20,283,392 bytes, S 401, D 1,
K 12.856051 and 55.455380 machine points. C's prior best was 51.737524;
the increase was 3.717856 points. The resulting score exceeded the registered
target of 53.4.

At 05:56:48 UTC, the board displayed 57.235 overall and rank 39. A, B and D
reran with unchanged sources and retained their previous bests. These are
live-board results, not final judging, an award or payment.

## Vector field search: insufficient incremental gain

Candidate v18 added a runtime-guarded AVX2 search for the first comma or quote,
with scalar tails and a non-AVX2 fallback. Text decoding copied literal spans
and handled doubled quotes between them; the two per-record numeric delimiter
scans used the same search. Membership, framing and worker counts were unchanged.
The older v4 segment decoder used scalar searches on the former serial pipeline;
its unsuccessful result remains separate.

Nine private Rust tests and all 15 correctness groups passed, including
unaligned vector boundaries, missing delimiters, tails, escaped text spanning
block boundaries, wide CSV grammar and reserved tiny seed 48. One stable
five-pair image batch measured 0.148895 s for accepted v17 and 0.146542 s for
v18: a speed ratio of 1.016060. The native peak-RSS ratio was 0.983443, while
the source-line proxy grew from 393 to 426. The incremental K-ratio proxy was
0.999211, below the fixed 1.08 gate; speed also missed the 1.18 gate.

The candidate was not submitted, and accepted v17 was restored. This batch
shows insufficient gain for the registered threshold, not an execution failure.
Its local timing, RSS and source-line proxies are not official measurements.
Further work needs current pipeline evidence and a distinct mechanism.

## Inline membership with worker-side framing

Candidate v20 combined accepted v17's exact inline-name membership with v15's
worker-side record framing. The producer emitted complete batches, and workers
reused vectors to locate individual records. The six-worker configuration,
64 shards, 64 KiB batch target and 1 MiB reader were unchanged. The earlier v15
result remains a failed experiment on the older baseline.

Nine private Rust tests and all 15 correctness groups passed, including reserved
tiny seed 51 and the published sample and full-output hashes. One stable
five-pair image batch measured 0.150613 s for v17 and 0.133035 s for v20, a speed
ratio of 1.132135. The native peak-RSS ratio was 1.003224. The source-line proxy
grew from 393 to 417, and the K-ratio proxy was 1.053921. Speed and efficiency
missed the fixed gates of 1.18 and 1.08. These local proxies are not official
measurements.

The candidate was not submitted. Accepted v17 was restored, and the candidate
and measurements were retained. Further adoption tests need fresh evidence
and a distinct mechanism; this result does not close the competition path.

## Borrowed names and fused numeric fields

Candidate v22 borrowed contiguous name values from each input row and used a
reusable decoded buffer only for doubled-quote escapes. It also combined the
id and region-id delimiter scans with decimal conversion. Note and amount
parsing, framing, membership and worker counts were unchanged.

Nine private Rust tests and all 15 correctness groups passed, including borrowed
slice identity, decoded-byte equivalence, numeric boundaries and reserved tiny
seed 54. One stable five-pair image batch measured 0.151709 s for accepted v17
and 0.147769 s for v22: a speed ratio of 1.026668. Native peak RSS had a ratio
of 1.004822, and the source-line proxy grew from 393 to 426. The K-ratio proxy
was 1.000103. Speed and efficiency missed the fixed 1.18 and 1.08 gates.

The candidate was not submitted; accepted v17 was restored. These local
measurements do not establish an official score. Current CPU samples justified
testing the work reduction, but did not predict its end-to-end benefit.

## Shared input blocks: no end-to-end improvement

Candidate v23 read into reusable 1 MiB blocks. Batches contained within one block
shared immutable byte views; batches crossing a block boundary retained owned
carry storage. A bounded, nonblocking pool received buffers only after their
last reader finished. Logical 64 KiB packet boundaries, row order, framing,
parsing, membership and the six-worker limit were unchanged.

Ten private Rust tests and all 15 correctness groups passed, including reserved
tiny seed 57. The ownership fixture preserved exact packet bytes and boundaries:
2,556,452 of its 2,736,709 input bytes used shared views. Other checks covered
last-reference recycling, a full recycle queue, short and interrupted reads,
a record spanning more than two blocks, and EOF without a final terminator.

One stable five-pair image batch measured 0.146800 s for accepted v17 and
0.151270 s for v23, a speed ratio of 0.970453. The native peak-RSS ratio was
1.039437. Source-line proxies were 393 and 455, giving a K-ratio proxy of
0.956279. Speed and efficiency missed the fixed 1.18 and 1.08 gates. Reducing
byte copies did not yield the required whole-process gain in this batch.

The candidate was not submitted, and accepted v17 was restored. These local
measurements do not establish an official score or identify which remaining
phase limited throughput. The frozen candidate and results are retained.

</details>
