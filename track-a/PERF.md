# Track A: performance and validation

Snapshot: 2026-10-31. This report describes the accepted source in repository
commit `72deb7d7d2abc47fa32758336a67fcacaa8c3b49`.

## Recorded official result

The best recorded live-board result was **61.431366 machine points**, from
`run-8ed375c52049` on commit `a19af6b27d282acc02b43cc963b83825081beadf`,
completed at 2026-10-31 03:13:53 UTC.

| Official axis | Value |
|---|---:|
| T: process time | 0.130583 s |
| M: cgroup memory peak | 49,778,688 bytes |
| S: source size | 254 |
| D: dependency axis | 1 |
| K: coefficient | 16.928845 |

All five board gates passed. This run accompanied a Track D restoration; A's
source had not changed. The earlier run that introduced the accepted A algorithm,
`run-5bc4a01a8486`, scored 57.856284. The later same-source increase is a recorded
measurement, not evidence of another algorithmic improvement. The best value was
reconfirmed at 08:02 UTC. Own-track observations and run IDs are in
[MEASUREMENTS.json](../MEASUREMENTS.json); these are practice scores, not final
judging or an award.

## What the implementation retains

The [implementation notes](README.md) explain the exactness arguments. The main
choices are disjoint byte-range readers, lazy endpoint/hour groups and exact
latency sorting. Endpoint scanning also computes a fingerprint; a fingerprint
match is always checked against the complete endpoint bytes. Hash collisions
therefore cannot combine different groups.

Output workers own contiguous blocks of sorted endpoints. Each formats one hour
at a time and sends its buffer through a zero-capacity channel. The writer receives
in hour-major, endpoint-block order. Completed latency vectors are released, while
all duplicate values remain available until their group's percentiles are computed.
Reading and formatting are parallel; merging and final ordered writes are serial.

## Local evidence for the accepted combination

Fused endpoint interning and bounded output each missed their standalone adoption
gates. A separately registered experiment tested their combination against the
then-accepted implementation; their individual speed ratios were not combined to
predict a result.

Five interleaved full-input pairs measured median command times of **0.167603 s**
and **0.125564 s**, a **1.334806×** ratio. Maximum process RSS was 39,972 and
39,912 KiB. The time/RSS/source-line coefficient proxy was 1.120126. All fixed
adoption gates passed. These measurements used an otherwise idle Intel Core
i5-12500H running Ubuntu 24.04 under WSL2, with six workers and Rust release
optimization. Each process read the input, aggregated it and wrote the full output.

The local source-line proxy was 218; the official S was 254. Process RSS is also
different from the board's cgroup memory peak. Neither proxy predicts the official
coefficient exactly. The first official result improved the previous best by
0.418766 points but missed the registered 58.5-point target. The valid improvement
was retained and the target miss was recorded.

## Correctness checks

The accepted combination passed 53 existing checks and four additional property
tests, including forced fingerprint collisions, endpoint boundaries, output ordering
and cancellation after a write error. Fresh tiny seeds 34 and 35 passed before
reserved seed 36 was generated. Shard-boundary fixtures check that each complete
line is assigned once, including long lines and exact boundary starts.

Published sample/full outputs matched exactly, including a full run in the
offline-built, unprivileged container with a read-only root and no network.
The expected full seed-1 `summary.txt` SHA-256 is:

`495ca09682b9673ef836a506496177680bb4a3af54255174befddb1f83481cd3`

Build and reproduction commands are in the [repository README](../README.md#build-and-check).

## Later trials and remaining limits

Two endpoint-storage trials were correct but did not meet their fixed 1.15× speed
and 1.06× coefficient-proxy requirements. Each used five paired pinned-image runs;
the table reports middle-three mean process times, excluding container startup.

| Candidate | Baseline → candidate | Speed ratio | K proxy |
|---|---:|---:|---:|
| Inline head IDs and dense collision links | 0.126725 → 0.121404 s | 1.043829 | 1.019183 |
| Inline endpoint bytes | 0.128257 → 0.133270 s | 0.962387 | 0.974872 |

Both passed their RSS limits. Neither was submitted; the accepted source was
restored. The measurements do not isolate allocation or cache effects.

Two current-source profiles placed endpoint comparison, fingerprint lookup,
hour lookup and latency insertion among the recurring CPU sample locations.
These are sampling observations, not wall-time fractions or measured memory stalls.
Group capacity, serial merging and ordered writes remain limits to investigate;
no further gain is assumed from those observations.

## Record and assistance

The [complete experiment log below](#complete-experiment-log) preserves the earlier performance log unchanged,
including rejected candidates, same-source reruns, timing repairs and thresholds.
Its references to a “current” or “pending” result describe the state when that
entry was written. This report identifies the accepted snapshot above.

Codex implemented and validated the changes.

## Complete experiment log

<details>
<summary>Open the full chronological record, including rejected trials and measurement repairs</summary>

# Track A: performance log

## Parallel byte-range candidate — local comparison, 2026-10-30

The accepted serial source's private instrumentation reached output at
0.409214 s and finished at 0.459818 s, with the published full output hash.
That measurement put roughly 89% of the local run before output. This candidate
divides the file into six line-aligned ranges and merges exact group data before
the existing sort-and-output stage. It keeps the whole-file input off the heap,
but duplicates the workers' interners and group-slot arrays.

Three paired runs used the same idle Victus CPU, interleaved as
old/new/new/old/old/new. Each timed process read the input, parsed and aggregated
it, merged where needed, sorted groups and wrote the complete output.

| Public seed 1 data | Serial median wall time | Parallel median wall time | Serial maximum peak RSS (KiB) | Parallel maximum peak RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 0.05 s | 0.04 s | 11,716 | 37,228 |
| Full | 0.45 s | 0.17 s | 27,372 | 53,100 |

Full wall times were 0.46, 0.45 and 0.42 s for the baseline, and 0.18, 0.16 and
0.17 s for the candidate: a median speed ratio of about 2.65, with 1.94 times
the peak process RSS. The sample's memory ratio was larger, about 3.18; fixed
per-worker structures matter more at that scale. These are local wall-time and
process-RSS observations, not official T or cgroup M measurements.

A simple nonblank, noncomment source-line count grew from 86 to 166; it is not
the board's formatted S count. Combining the full local timing, RSS and source
proxies gives a K ratio of about 1.291. The preregistered full-data gate required
a speed ratio of at least 1.50, an RSS ratio no greater than 2.50 and a K proxy
ratio of at least 1.10. All three passed without a parameter sweep.

Correctness checks completed before any official push:

- Every possible byte cut in a fixed multi-hour example, including empty edge
  ranges, plus many-way partitions merged in reverse order. The expected output
  was specified independently and included duplicate requests.
- Forty-five fixtures at 1, 2, 6 and 12 workers against the independent heap-based
  oracle: empty and single-line files, the worked and edge examples, all 500 legal
  status values, duplicates, and 40 shuffled random cases. The edge example covers
  nearest-rank rounding, month rollover and endpoint byte order.
- Tiny seeds 7 and 8, followed by reserved seed 9 with the source frozen, at all
  four worker counts. Both the official reference and the heap oracle agreed.
- Published sample/full input and output hashes, separate 1- and 2-worker runs,
  and every six-worker timed run.
- Exact full output in the offline-built Docker image as uid 65534, with no
  network, read-only root, 6 CPUs, 8 GB memory, no swap and a 1 GB tmpfs.
  The image was 30,071,197 bytes.

Tested parallel candidate `src/main.rs` SHA-256:
`7c083c9c6a59fa04807666359936d4d1e7c00c1c11721b88c243536294b32726`.
Codex implemented and validated the change. Its official result was not available
when this comparison was recorded; the private scoring inputs remain unseen.

## Live-board history

The first candidate replaces Python timestamp parsing and object-heavy groups
with a single-threaded Rust byte parser and compact integer latency vectors.
The [official live board](https://speeddemon.dev/board/) recorded the first run
on 2026-10-30, starting at 20:12:40 UTC and ending at 20:12:41 UTC.

| Run | Commit | Result | T (s) | M (bytes) | K | Machine points |
|---|---|---|---:|---:|---:|---:|
| `run-713a55a5de05` | `8a1a566` | Correct | 0.671122 | 37,187,584 | 9.188717 | 48.162744 |
| `run-6ab97c79674a` | `0a3e541` | Correct, unchanged A source | 0.671838 | 37,072,896 | 9.189495 | 48.164582 |
| `run-9e782f477ef0` | `bc07ea2` | Correct, unchanged A source | 0.670863 | 36,962,304 | 9.201667 | 48.193324 |
| `run-4706ea04e92c` | `82216f0` | Correct, unchanged A source | 0.679358 | 37,076,992 | 9.138291 | 48.043249 |
| `run-c8dacaf730a8` | `17838c7` | Correct, unchanged A source | 0.676361 | 37,179,392 | 9.153464 | 48.079274 |

Check, build, correctness, timeout and sandbox gates all passed. The board
reported S = 94 and D = 1, and displayed rank 48 in Track A at 20:14:55 UTC.
These are live-board observations, not final five-run measurements. With only
A and D scored at that time, the team did not yet have an overall score.

The second run accompanied the first Track C implementation push. A's source
was unchanged; all five gates passed, with S = 94 and D = 1 again. It ran from
20:21:21 to 20:21:22 UTC. At 20:22:29 UTC the team had three scored tracks and
an overall 40.591, rank 40. The small A measurement change is a same-source
observation, not an optimization claim.

The next run accompanied the first Track B implementation push, with unchanged
A source. It ran from 20:37:52 to 20:37:53 UTC; all five gates passed
and S = 94, D = 1 were unchanged. At 20:39:16 UTC, the team's overall display was
40.969, rank 40. This same-source measurement is not an algorithmic improvement.

The next run accompanied D's bidirectional-search update with unchanged A
source. All five gates passed; S = 94 and D = 1 were unchanged. Its latest
48.043249 points were below the prior best 48.193324, which the page
retained. At 20:51:13 UTC the overall display was 42.042, rank 38. This is a
same-source control, not an algorithmic regression or improvement claim.

## Local validation — 2026-10-30

Machine: Intel Core i5-12500H, Ubuntu 24.04 under WSL2, Rust 1.99.0 and Python 3.12.
Both programs ran on the same otherwise idle development machine. Each timed
program read the input, aggregated its groups and wrote the complete output.
GNU `time` recorded one wall-time and peak-process-RSS observation per program
and scale; these are not official five-run T or cgroup `memory.peak` M values.

| Public seed 1 data | Python reference wall time | Rust wall time | Reference peak RSS (KiB) | Rust peak RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 2.02 s | 0.05 s | 67,328 | 11,628 |
| Full | 18.60 s | 0.43 s | 218,016 | 27,240 |

The full-data pair showed a wall-time speedup of about 43.3×. This single pair
does not establish repeatability or predict the board coefficient. The local
reference also uses Python 3.12, while official scoring uses Python 3.13.

Correctness checks passed:

- Empty input, the official worked example and the official edge-case fixture,
  checked against literal expected output and an independent heap-selection oracle.
- All 500 legal status values, duplicate requests and reversed input order.
- Tiny development seeds 4 and 5 against both the reference and the heap oracle.
  Tiny seed 6 was generated after these checks passed, with no subsequent source edit.
- Published sample and full seed 1 input and output SHA-256 values.
- Full-data output from the offline-built image as uid 65534, with no network,
  read-only root, an 8 GB memory limit, no swap and a 1 GB tmpfs. The final image
  was 30,043,812 bytes. This check did not use the official scoring harness.

Full output SHA-256:
`495ca09682b9673ef836a506496177680bb4a3af54255174befddb1f83481cd3`.

Tested `src/main.rs` SHA-256:
`8c002e10e31cd299d190f30cc13538ec4b2ff2f442780531d619b6cb29d88349`.

## Interpretation

The reference parses a full timestamp for every request and stores Python
integer objects. This candidate groups by the fixed UTC-hour prefix and stores
32-bit latency values. It still sorts every group's latencies. No ablation was
run, so the measurements cover these changes together and do not assign the
speedup to any one of them. The official live-board result is recorded above;
no final score or award is claimed.

## Same-source observation with the C bitset push

Commit `17838c7` changed C; this track's source was unchanged. Run
`run-c8dacaf730a8` passed all five gates and recorded 48.079274
machine points. The board retained the prior best 48.193324.
At 21:04:13 UTC, the overall display was 44.759, rank 38.

## Same-source observation with the B shard push

Commit `57f452c` changed B; this track's source was unchanged. Run
`run-332b9702f5a7` passed all five gates and recorded 48.160952
machine points. The board retained the prior best 48.193324.
At 21:36:09 UTC, the overall display was 47.174, rank 36.

## Same-source observation with the B automaton push

Commit `a3a4156` changed B; this track's source was unchanged. Run
`run-78501d9c5869` passed all five gates and recorded 48.00429
machine points. The board retained the prior best 48.193324.
At 21:50:14 UTC, the overall display was 48.330, rank 35.

## Same-source observation with the D radix push

Commit `47a7c57` changed D; this track's source was unchanged. Run
`run-5bc81b8f4f7d` passed all five gates and recorded 48.208906
machine points. This is a new same-source best over 48.193324; it does not establish
a new A algorithm improvement.
At 22:11:03 UTC, the overall display was 48.815, rank 34.

## Parallel-range candidate — official observation

Commit `014bdb7` ran as `run-5d356b404ed7` from 22:22:43 to 22:22:44 UTC
on 2026-10-30. Check, build, correctness, timeout and sandbox gates all passed.
The feed recorded T = 0.206909 s, M = 59,494,400 bytes, S = 174, D = 1,
K = 13.735267 and 56.891855 machine points. This exceeds the registered 51.0-point
gate and the previous A best of 48.208906 by 8.682949 points (calculated).
Compared with that best run, T fell about 69.0% and M rose about 60.0%.

At 22:23:46 UTC, the page displayed 51.600 overall, rank 35. The best three were
A = 56.891855, B = 49.643708 and D = 48.644790. B and C retained their earlier
best scores. D improved slightly in the same dispatch, with unchanged source;
that difference is not an A algorithm improvement. These are live board
observations, not the final five-run measurement, judged total or award.

## Lazy endpoint-hour groups — local comparison

The accepted parallel-range version allocated 72 `Group` slots for every
endpoint in every worker. The candidate starts each vector empty and grows it
through the hour ID being written. It also skips empty source groups while
merging and missing slots while producing output. The line ownership rule,
full-key merge, raw latency values and duplicate handling are unchanged.

The frozen candidate passed every byte-cut and reverse-merge test, 47 oracle
fixtures (including sparse and permuted 72-hour inputs), and tiny seeds 10, 11
and 12 at one, two, six and twelve workers. Seed 12 was reserved until 10 and 11
passed. The official reference and independent heap oracle agreed. Published
sample/full hashes matched at one, two and six workers. An offline, read-only,
unprivileged Docker run also matched the full output hash.

Five interleaved baseline/candidate pairs ran on the same Victus CPU environment
with six workers. `/usr/bin/time` reports hundredths of a second, so equal times
here do not establish identical runtimes.

| Input | Baseline median time | Candidate median time | Baseline peak RSS | Candidate peak RSS |
| --- | ---: | ---: | ---: | ---: |
| Sample | 0.04 s | 0.04 s | 36,988 KiB | 22,356 KiB |
| Full | 0.18 s | 0.18 s | 52,972 KiB | 39,784 KiB |

Full peak RSS fell about 24.9%. The simple nonblank-code-line count rose from
166 to 177; it is not the board's source-size measure. The rough K-ratio proxy
was 1.048788, including that source-size penalty. The predeclared local gate
required RSS at most 80% of baseline, speedup at least 0.90 and proxy at least
1.03. It passed. These measurements are local evidence; official points must
come from the board.

## Lazy endpoint-hour groups — official observation

Commit `86a95c8` ran as `run-812e64eace9b` from 22:38:11 to 22:38:12 UTC
on 2026-10-30. Check, build, correctness, timeout and sandbox gates all passed.
The feed recorded T = 0.206905 s, M = 50,114,560 bytes, S = 185, D = 1,
K = 14.084789 and 57.437518 machine points. A improved by 0.545663 points from
56.891855, but missed the predeclared 57.5-point threshold by 0.062482. That
score-target hypothesis failed; the upload, runtime and correctness checks did
not. The valid new best is retained without a timing-only retry.

Official peak memory fell about 15.8%, less than the local 24.9% reduction.
Official runtime was nearly unchanged (0.206909 to 0.206905 s), while S rose
from 174 to 185. The local proxy overstated the eventual official gain; it was
not used as a substitute for this readback.

At 22:38:49 UTC, the board displayed 51.765 overall, rank 35. B, C and D had
unchanged source and lower measurements in the same dispatch; their earlier
best scores remained on the board. These are live machine results, not final
judging, an award or payment.

## Same-source observation with the D compact-storage push

Commit `e0378c5` changed D; this track's source was unchanged. Run
`run-541aadbd6ef0` passed all five gates and recorded 54.641618
machine points. The board retained the earlier best 57.437518.
At 22:50:46 UTC on 2026-10-30, the overall display was 51.884, rank 35.
This rerun does not establish a new algorithm improvement, final judging or an award.

## Same-source observation with the D CSR-index push

Commit `2f5cdaf` changed D; this track's source was unchanged. Run
`run-413a14a0a34d` passed all five gates and recorded 57.421273
machine points. The board retained the earlier best 57.437518.
At 23:07:03 UTC on 2026-10-30, the overall display was 52.170, rank 34.
This rerun does not establish a new algorithm improvement, final judging or an award.

## Same-source observation with the popped-vertex meeting push

Commit `b94f7ff` changed D; this track's source was unchanged. Run
`run-b4d8e3a63f12` passed all five gates and recorded 57.333396
machine points. The board retained the earlier best 57.437518.
At 23:53:09 UTC on 2026-10-30, the overall display was 52.240, rank 35.
This rerun does not establish a new algorithm improvement, final judging or an award.


## Exact quantile selection — local rejection, 2026-10-31

This candidate selected the exact p50, p95 and p99 ranks from successive
partitions of each large group's latency vector. Groups of at most 32 values
kept the existing full sort. Parsing, shards, grouping and merging were unchanged.
The source passed 53 host, heap-oracle, range and published-hash checks, plus
private quantile tests over boundary sizes, duplicates, reversed and extreme
values. Seeds 13 and 14 passed before reserved seed 15; the offline container
matched the full expected output.

The first timing report was invalid. CPython's timed child wait polled at up to
50 ms intervals, adding a quantized delay to the parent elapsed clock. The
apparent 1.2894 speed ratio was therefore not accepted. The original data was
retained, and only the timing procedure was repaired: the same frozen binaries
were timed with blocking child waits under the outer run timeout.

Five corrected interleaved full pairs had median elapsed command times of
0.165261 s for the accepted code and 0.156303 s for the candidate, a
1.05731 speed ratio. Maximum process RSS was 39,964 and 39,916
KiB. These measurements include the GNU time wrapper; they are not official T
or cgroup memory.peak. The combined time/memory/source-size proxy was
1.01135, below the fixed 1.06 requirement; the speed ratio
also missed 1.15. No official push followed. The accepted source was restored,
and the candidate was not retuned. Codex implemented and checked this change.


## Borrowed block lines — local rejection, 2026-10-31

The candidate passed complete lines directly from a 1 MiB read buffer to the
aggregator. Only lines split across buffers were copied. Shard ownership and
all grouping, percentile and output rules were unchanged. Safe UTF-8 validation
and the standard-library newline splitter added no dependency.

All 53 existing checks passed, as did two private tests covering every byte
cut, reversed merges, 1–256-byte buffers, maximum-length endpoints and EOF.
Seeds 16 and 17 passed before reserved seed 18. The offline, unprivileged,
read-only container matched the full published output hash.

Five interleaved full pairs used blocking child waits under an outer timeout.
Median command times were 0.182799 s for the accepted code and 0.183951 s for
the candidate; maximum process RSS was 39,936 and 39,968 KiB. The speed ratio
was 0.993739 and the time/memory/source-size proxy was 0.974988, below the
fixed 1.15 and 1.06 gates. The measurements do not isolate validation, scanning
or copying costs. No official push followed; the accepted source was restored.
Codex implemented and checked this change.


## Calendar-hour lookup — local rejection, 2026-10-31

The input contract fixes 72 hours across October 30–November 2. This candidate
used a 72-byte table to map each observed day/hour to its compact hour ID,
removing the per-line hour-key hash. It retained the full observed timestamp
keys, sparse groups, original reader, sorting and output. It did not include
the rejected block reader.

All 53 existing checks passed, along with an exhaustive 72-hour ID/key test
in three encounter orders. Seeds 19 and 20 passed before reserved seed 21;
the offline container matched the full published output.

Five interleaved full pairs gave median command times of 0.169770 and
0.159967 s, a 1.061281 speed ratio. Maximum process RSS was 39,956 and
39,928 KiB. The combined time/memory/source-size proxy was 1.013147. Both
the 1.15 speed gate and 1.06 proxy gate were missed. The accepted source was
restored without an official push or date-arithmetic tuning. Codex implemented
and checked this change.


## Direct decimal rows — local rejection, 2026-10-31

The candidate built each output row in one reusable byte vector. A 20-byte
stack scratch encoded each of the six unsigned decimal fields, and the row
was written once to the existing buffered writer. The original reader, hour
hash map, sparse groups and full-sort quantiles were unchanged.

All 53 existing checks passed. A separate decimal test compared the helper
with the standard formatter for 0–100,000, powers of ten and their neighbours,
u32/u64 limits and 10,000 deterministic u64 values. Seeds 22 and 23 passed
before reserved seed 24. The offline container matched the full output hash.

Five interleaved full pairs gave median command times of 0.167852 and
0.151050 s. The 1.111234 speed ratio corresponds to about 10.0% less elapsed
time. Maximum process RSS was 39,960 and 39,956 KiB; the simple source-line
proxy grew from 177 to 196. The combined proxy was 1.038171, below the fixed
1.06 gate, and speed also missed 1.15. No official push followed. The accepted
source was restored; no digit-batching or buffer tuning was attempted. Codex
implemented and checked this change.


## Parallel endpoint blocks — local rejection, 2026-10-31

This candidate sorted endpoints by their actual bytes and gave contiguous
blocks to scoped workers. Each worker sorted its groups and formatted rows
into separate hour buffers. The main thread joined those buffers in hour and
endpoint order. Latency arrays were released after their rows were formatted.

All 53 existing checks and the additional sparse-hour, endpoint-prefix and
one-shot consumption test passed. Seeds 25 and 26 passed before reserved
seed 27; the offline container matched the full output hash.

Five interleaved full pairs gave median command times of 0.161605 and
0.130207 s: a 1.241137 speed ratio, or 19.4% less elapsed time. Maximum
process RSS rose from 39,964 to 47,184 KiB, a 1.180663 ratio. The combined
time/memory/source-size proxy was 1.068820. Speed and proxy passed their
fixed gates, but peak memory exceeded 1.05. The source was restored and
no official push followed. Codex implemented and checked this candidate.


## Rendezvous hour buffers — local rejection, 2026-10-31

This follow-up replaced retained whole-output buffers with a zero-capacity
channel for each endpoint block. A worker sorted and formatted one hour,
transferred its buffer to the writer, then continued. The writer received
buffers in hour and endpoint-block order. Completed latency arrays were freed.

All 53 existing checks and two additional property tests passed, including
endpoint-prefix order, sparse hours and cancellation after a sink error.
Seeds 28 and 29 passed before reserved seed 30. The offline container matched
the full published hash.

Five interleaved full pairs gave median command times of 0.164894 and
0.145468 s, a 1.133544 speed ratio, or 11.8% less elapsed time. Maximum
process RSS was 39,940 and 39,920 KiB, within the 1.05 ratio limit. The
combined time/memory/source-size proxy was 1.049430. Speed and proxy missed
the fixed 1.15 and 1.06 gates. The accepted source was restored with no
official push. Codex implemented and checked this candidate.


## Accepted-source CPU attribution, 2026-10-31

A private, byte-identical copy of the accepted source was built with optimized
debug information. Two process-scoped software profiles each ran the full input
five times at 499 Hz. All outputs matched. Exact ELF address resolution gave
1,341 and 1,360 samples. The endpoint lookup source site accounted for 13.05%
and 13.75%; endpoint delimiter scanning accounted for 10.81% and 9.12%.
Unresolved addresses and other libraries accounted for 19.31% and 19.63%.
These are exclusive CPU sample locations, including wrapper and profiling
effects, not elapsed-time shares or a benchmark improvement.

## Fused endpoint interning — local rejection, 2026-10-31

The candidate detected the endpoint boundary and folded its bytes into a
fingerprint in one word-wise pass. Fingerprint buckets compared complete
endpoint bytes before reusing an ID. The original reader, hour map, groups,
full-sort quantiles and serial output were retained.

All 53 checks passed, along with tests for endpoint lengths 1–125, word
boundaries, suffix independence and deliberately forced hash collisions.
Seeds 31 and 32 passed before reserved seed 33; the offline container matched
the full output hash. Median command time fell from 0.161142 to 0.148478 s
over five interleaved pairs. The speed ratio was 1.085296, RSS ratio 1.000100
and combined proxy 1.022838. The standalone candidate missed the fixed
1.15 speed and 1.06 proxy gates and was not submitted.

## Cross-stage pipeline — local validation, 2026-10-31

One separately registered candidate combined the fixed fused endpoint handling
with the fixed rendezvous-hour output design. Neither standalone rejection
was reversed. The integrated binary was checked against the original accepted
baseline with the same speed, memory and source-size gates. No hash constants,
channel capacities or worker counts were tuned for the comparison.

All 53 existing checks and four additional property tests passed, including
forced collisions, endpoint boundaries, output ordering and sink-error
cancellation. Seeds 34 and 35 passed before reserved seed 36. The offline,
unprivileged container matched the full published output hash.

Five interleaved full pairs gave median command times of 0.167603 and
0.125564 s, a 1.334806 speed ratio, or 25.1% less elapsed time. Maximum
process RSS was 39,972 and 39,912 KiB. The source-size-aware proxy was
1.120126. All three fixed gates passed. The proxy uses nonblank source lines
and is not the official scorer's SCC count. These are local results; no
official points are inferred. Codex implemented and validated the changes.


## Official cross-stage result, 2026-10-31

Commit `0f25f4a` was submitted once. Official run `run-5bc4a01a8486`
passed all five gates: T = 0.181018 s, M = 50,114,560 bytes, S = 254,
D = 1 and K = 14.359049. Its 57.856284 machine points became the new
Track A best, 0.418766 above 57.437518. The registered 58.5 target was
missed by 0.643716; this was a threshold miss, not a failed execution.

The official source count was 254; the local nonblank-line proxy was 218.
Local timing and source counts therefore did not predict the official gain
exactly. The valid improved source is retained. No unchanged retry was made.
At 02:16:29 UTC, the overall display was 52.515, rank 37. D also recorded
a new best on unchanged source, so that change is not attributed to this
Track A algorithm. The board is practice scoring, not an award or payment.


## Current-source CPU attribution

A private debug build of the accepted fused-interning and rendezvous-output
source was sampled twice on 2026-10-31. Each profile covered five full-input
executions with six workers. The sample control, full control and all ten
profiled outputs matched their published hashes. Production source stayed
unchanged.

The two profiles captured 1,171 and 1,162 process CPU samples. Both placed
the same four source sites among their top five: endpoint byte comparison
(109 and 104 samples), fingerprint-table entry lookup (58 and 51), hour lookup
(69 and 63), and latency insertion (42 and 36). Addresses were resolved through
the exact ELF executable segments and its debug information.

These are exclusive CPU sample locations, including wrapper and runtime
overhead, not wall-time shares or proof of memory stalls. Lost samples were
not measured. The result supports investigating the current dictionary
representation; it does not establish a speedup or an official score gain.

## Endpoint storage experiments, 2026-10-31

Two separately registered candidates changed the endpoint dictionary. The first
stored the fingerprint's head ID in the hash table and collision links in a dense
array. The second also stored each endpoint's bytes inline in a contiguous record,
using the specification's 125-byte maximum. Both retained full-byte equality on
hash collisions and left parsing, grouping, quantiles and output scheduling unchanged.

Each candidate had its own fixed five-pair comparison against the accepted source,
using pinned images built from the submission Dockerfile and the published child timer on Victus.
The table reports the mean of the middle three times from each set of five.

| Candidate | Baseline time | Candidate time | Speed ratio | Peak process RSS, baseline → candidate | K proxy |
|---|---:|---:|---:|---:|---:|
| Inline head IDs | 0.126725 s | 0.121404 s | 1.043829 | 39,896 → 39,840 KiB | 1.019183 |
| Inline endpoint bytes | 0.128257 s | 0.133270 s | 0.962387 | 39,912 → 39,180 KiB | 0.974872 |

The fixed gates required at least 1.15× speed and 1.06× K proxy, with a process-RSS
ratio no greater than 1.05. Both missed the efficiency gates. All 53 recorded check
groups passed in each run; the private Rust suites contained seven and eight tests respectively.
Fresh seeds 109–111 and 112–114 included a reserved final seed in each experiment.
Long forced-collision chains, all valid endpoint lengths, shard boundaries and
published output hashes were checked.

Neither candidate was submitted. The accepted source and native executable were
restored. These whole-process measurements do not isolate cache or allocation costs;
the RSS and source-line inputs to K remain local proxies, not official scoring data.

</details>
