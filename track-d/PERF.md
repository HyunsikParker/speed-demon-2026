# Track D: performance log

## Radix-queue candidate — local comparison, 2026-10-30

A fresh instrumented copy of the accepted bidirectional solver reached graph
readiness at 0.445 s and finished at 1.175 s. The remaining roughly 0.730 s was
the concurrent search interval, not the sum of worker times. This prompted a
queue-only change; CSR construction, distance arrays, bidirectional alternation
and the default six workers are unchanged.

Three baseline/candidate pairs ran in interleaved order on the same idle Victus
CPU, including input parsing, graph construction, all queries and output.

| Public seed 1 data | Binary-heap median | Radix-heap median | Binary-heap maximum peak RSS (KiB) | Radix-heap maximum peak RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 0.07 s | 0.06 s | 36,784 | 39,136 |
| Full | 1.14 s | 0.95 s | 291,652 | 294,216 |

Full wall times were 1.14, 1.13 and 1.15 s for the baseline, and 0.94, 0.95 and
0.95 s for the candidate. The median speed ratio was 1.20, with about 0.9% more
peak process RSS. These local measurements are not official T or cgroup M.
A source-line proxy grew from 197 to 244 lines; it is not the board's formatted
S count. Combining those local proxies gives a K ratio of about 1.059. The fixed
local gate required a speed ratio of at least 1.15, an RSS ratio no greater than
1.12, and a K proxy ratio of at least 1.05. It passed without further tuning.

Before any official push, correctness checks passed:

- Independent ordered-multiset tests for queue minima, duplicate entries,
  equal keys, every bit boundary, zero increments, `u64::MAX` and repeated resets.
- All seven host cases with 1, 6 and 12 workers; all-pairs Bellman–Ford agreement
  on 103 adversarial and random directed graphs.
- Tiny seeds 16 and 17, then reserved seed 18 with the source frozen. Every host
  query category was present, and both the reference and Bellman–Ford agreed.
- All published sample/full input and output hashes. Separate 1- and 2-worker
  runs and all six-worker timed runs matched the expected output.
- Exact full output in the offline-built container, uid 65534, with no network,
  read-only root, 6 CPUs, 8 GB memory, no swap and a 1 GB tmpfs. The image was
  30,047,847 bytes.

Tested radix candidate `src/main.rs` SHA-256:
`a6d8cd2155b757d4b72c15534e5cfc01738b3379cd09e8f7f7ae888a5d228e58`.
The official private inputs remain unseen. No official radix-candidate result
was available when this comparison was recorded.

## Live-board history

The first candidate is a Rust CSR implementation with epoch-tagged distance
arrays and, by default, up to six independent query workers.

| Run | Commit | Result | T (s) | M (bytes) | K | Machine points |
|---|---|---|---:|---:|---:|---:|
| `run-d2234e76683a` | `06d5a2c` | Correct | 1.689332 | 222,765,056 | 7.144156 | 42.697548 |
| `run-f0f2ac9f797d` | `8a1a566` | Correct, unchanged D source | 1.725366 | 222,875,648 | 7.068459 | 42.466237 |
| `run-44b5bb312166` | `0a3e541` | Correct, unchanged D source | 1.701160 | 222,834,688 | 7.118832 | 42.620437 |
| `run-518ec8eea2ab` | `bc07ea2` | Correct, unchanged D source | 1.795418 | 222,773,248 | 6.929828 | 42.036124 |
| `run-689c61dfbf24` | `82216f0` | Correct, bidirectional search | 1.005084 | 299,241,472 | 8.372574 | 46.142951 |
| `run-e367e92f0b70` | `17838c7` | Correct, unchanged D source | 0.913775 | 298,807,296 | 8.783481 | 47.183334 |

The [official live board](https://speeddemon.dev/board/) and its linked data feed
reported this result on 2026-10-30 at 15:30–15:31 UTC. The run started at 15:30:18
and ended at 15:30:20. Build, correctness, timeout and sandbox gates all passed.
The board reported S = 155 and D = 1; its displayed points round to 42.698.

The second run accompanied the first Track A implementation push and changed
only this log in Track D. It ran from 20:12:42 to 20:12:44 UTC; all five gates
passed and S = 155, D = 1 were unchanged. The linked feed reported the new
measurement, while the page still displayed 42.698 at 20:14:55 UTC. The slower
second timing is an observation of the same source, not a code regression test.

These are live-board observations, not the final five-run measurements or judged
track total. Only A and D were scored after the second push, so the team still
had no overall score.

The third run accompanied the first Track C implementation push, again with
unchanged D source. It ran from 20:21:25 to 20:21:27 UTC; all five gates passed
and S = 155, D = 1 were unchanged. The board retained the first run's 42.697548
as D's best history. With A, C and D scored, the team displayed an overall
40.591, rank 40, at 20:22:29 UTC.

The next run accompanied the first Track B implementation push, with unchanged
D source. It ran from 20:38:02 to 20:38:04 UTC; all five gates passed
and S = 155, D = 1 were unchanged. The page retained 42.697548 from the first run
as D's best history. At 20:39:16 UTC, the team's overall display was
40.969, rank 40. This same-source measurement is not an algorithmic improvement.

## Bidirectional candidate — local comparison, 2026-10-30

A private phase-timing copy of the accepted forward solver spent 0.413 s reading
and building the graph, and about 2.001 s answering queries, out of 2.414 s total.
Its output still matched the published full hash. This identified query work as
the first optimization target; these instrumented timings are not board scores.

The second candidate builds reverse CSR too, alternates exact forward/reverse
Dijkstra steps, and stops at a minimum-frontier-distance bound. Touched lists
replace epoch arrays so each worker can reuse both distance arrays. It still
uses six workers by default. The extra graph and frontier state cost memory.

The comparison used the same Victus CPU and compiler settings as the baseline.
Three runs of each version were interleaved in the order old/new/new/old/old/new,
with six workers. Every run read the full inputs, built its graph, answered all
queries and wrote the output; all output hashes matched. No tuning followed
these measurements.

| Public seed 1 data | Forward median wall time | Bidirectional median wall time | Forward maximum peak RSS (KiB) | Bidirectional maximum peak RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 0.06 s | 0.06 s | 22,524 | 36,644 |
| Full | 2.33 s | 1.11 s | 218,208 | 291,304 |

Full wall times were 2.33, 2.28 and 2.33 s for the baseline, and 1.11, 1.11 and
1.12 s for the candidate. The median speedup was about 2.10×, with roughly 33.5%
more peak process RSS. Sample times were equal at GNU `time`'s printed precision.
These process measurements do not establish official T, cgroup M or K. The
subsequent live-board result is recorded below; it is not the final five-run score.

Correctness checks for the second candidate passed:

- All seven official hand-written cases with 1, 6 and 12 workers.
- All pairs in 103 adversarial and randomly generated directed graphs, checked
  against independent Bellman–Ford distances. Cases included an incorrect
  first-meeting trap, one-way bridges, zero cycles and asymmetric large weights.
- Tiny seeds 13 and 14, followed by reserved seed 15 with the source frozen.
  Each included all host query categories and matched both the official
  reference and Bellman–Ford.
- Published sample/full output hashes and full input hashes.
- Full output in the offline-built container as uid 65534 with no network,
  read-only root, 6 CPUs, 8 GB memory, no swap and a 1 GB tmpfs. Image size:
  30,047,129 bytes. This was a runtime-contract check, not official scoring.

Tested second-candidate `src/main.rs` SHA-256:
`78a1c00a192e68253f6310fd9343b50d2650d793962db06d6af4790028152fc2`.
Codex implemented this candidate and performed the comparisons.

## Bidirectional candidate — official observation

The new D run, `run-689c61dfbf24`, ran from 20:50:53 to 20:50:55 UTC on
2026-10-30. Check, build, correctness, timeout and sandbox gates passed. The board
measured S = 205 and D = 1, T = 1.005084 s, M = 299,241,472 bytes and K = 8.372574.
Its 46.142951 machine points exceeded the prior best 42.697548 by 3.445403 points
(calculated from the two feed values, not a separate displayed field).

At 20:51:13 UTC, the team displayed 42.042 overall, rank 38, using the best three
track scores. A, B and C were same-source controls, and all three latest runs
were below their earlier bests. The local 2.10× time ratio did not carry over
unchanged: D's prior-best and new official times were 1.689332 and 1.005084 s.
The observed improvement includes higher memory use and source-size cost in K.
It is one live-board observation, not final judging or an award.

## Local validation — 2026-10-30

Machine: Intel Core i5-12500H, Ubuntu 24.04 under WSL2, Rust 1.99.0 and Python 3.12.
Both programs ran on the same otherwise idle development machine. Each timed
program read the input, built its graph, answered the queries and wrote its output.
GNU `time` recorded one wall-time and peak-process-RSS observation per program and
scale. These are not the official five-run T or cgroup `memory.peak` M measurements.

| Public seed 1 data | Python reference wall time | Rust wall time | Reference peak RSS (KiB) | Rust peak RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 3.45 s | 0.06 s | 93,464 | 22,500 |
| Full | 55.97 s | 2.42 s | 874,696 | 218,236 |

The full-data pair showed a wall-time speedup of about 23.1×. A single pair does not
establish repeatability or predict the board coefficient. The comparison also
uses local Python 3.12, while official reference scoring uses Python 3.13.

Correctness checks passed:

- The seven official hand-written cases with 1, 6 and 12 workers, plus a one-way
  cross-SCC regression with forward, reverse, same-node and duplicate queries.
- Public tiny seeds 1 and 2 against both the reference and an independent
  Bellman-Ford calculation. Tiny seed 3 was generated after those checks passed;
  the implementation was unchanged when it was checked.
- The published sample and full seed 1 input and output SHA-256 values.
- The full-data output from the offline-built Docker image, running as uid 65534
  with no network, read-only root, an 8 GB memory limit, no swap and a 1 GB tmpfs.
  The final image was 30,045,892 bytes. The official scoring harness was not used
  for this container check.

Full output SHA-256:
`a23dbdf0a3e49cc8901310a9ee1dc6a9a569a85297b6da75c2a7002cb2ed277e`.

Tested `src/main.rs` SHA-256:
`82b8b1f42c17e95dcd802a4c3c2d59486207faf48432fd67fab25dd1634aa48b`.

## Why this version changes the work

The reference allocates Python adjacency objects and a fresh distance dictionary
per query. CSR stores the edges contiguously, and workers reuse typed distance
and epoch arrays. The safe no-in-target check also avoids searching the whole
main component for that unreachable query type. No separate ablation was run,
so the timing above measures these changes together and cannot attribute the
speedup to any one of them.

The live-board score is recorded above; no final score, overall rank or award is
claimed. Codex implemented the candidate and ran the checks. The saved outputs,
timing observations and official board readback support the measurements here.

## Same-source observation with the C bitset push

Commit `17838c7` changed C; this track's source was unchanged. Run
`run-e367e92f0b70` passed all five gates and recorded 47.183334
machine points. This was a new best over 46.142951, but it is a same-source
measurement, not evidence of a new D algorithm improvement.
At 21:04:13 UTC, the overall display was 44.759, rank 38.

## Same-source observation with the B shard push

Commit `57f452c` changed B; this track's source was unchanged. Run
`run-9644ded2c069` passed all five gates and recorded 47.157433
machine points. The board retained the prior best 47.183334.
At 21:36:09 UTC, the overall display was 47.174, rank 36.

## Same-source observation with the B automaton push

Commit `a3a4156` changed B; this track's source was unchanged. Run
`run-0d50cd164bbb` passed all five gates and recorded 46.896596
machine points. The board retained the prior best 47.183334.
At 21:50:14 UTC, the overall display was 48.330, rank 35.

## Radix-queue candidate — official observation

Commit `47a7c57` ran as `run-171aac4335e7` from 22:10:39 to 22:10:40 UTC
on 2026-10-30. Check, build, correctness, timeout and sandbox gates all passed.
The feed recorded T = 0.750944 s, M = 301,494,272 bytes, S = 252, D = 1,
K = 9.376871 and 48.602898 machine points. This exceeds the registered 48.2-point
gate and the previous D best of 47.183334 by 1.419564 points (calculated).
Compared with that best run, T fell about 17.8% and M rose about 0.9%.

At 22:11:03 UTC, the page displayed 48.815 overall, rank 34. The best three were
B = 49.643708, D = 48.602898 and A = 48.208906. A's small new best came from
unchanged source; the same dispatch measured lower B and C scores than their
previous bests. These observations show timing variation and are retained in
the respective logs. The live board's best-history display is not the final
five-run measurement, judged total or award.

## Same-source observation with the A parallel-range push

Commit `014bdb7` changed A; this track's source was unchanged. Run
`run-d35ed8feb854` passed all five gates and recorded 48.64479
machine points. This is a new same-source best over 48.602898; it does not establish
a new D algorithm improvement.
At 22:23:46 UTC on 2026-10-30, the overall display was 51.600, rank 35.
These are live board results, not final judging or an award.

## Same-source observation with the A lazy-group push

Commit `86a95c8` changed A; this track's source was unchanged. Run
`run-0fde281b7ccd` passed all five gates and recorded 48.543152
machine points. The board retained the earlier best 48.64479.
At 22:38:49 UTC on 2026-10-30, the overall display was 51.765, rank 35.
This rerun is not evidence of a new algorithm, final judging or an award.

## Compact distances and earlier buffer release — local comparison

This candidate changes the memory use of both major phases. It frees graph input
bytes after parsing and before building the CSR arrays. Each search then keeps
small distances inline in a `u32` array, with separate unvisited and overflow
markers. Overflow values remain exact `u64` entries in a hash map. Queue keys,
path sums and answers retain 64-bit arithmetic. The radix queue and bidirectional
stopping rule are unchanged.

Four property tests covered radix ordering, duplicate keys, marker boundaries,
large sums, overflow-to-inline updates and search resets. Seven host examples
passed at one, two, six and twelve workers. On 107 adversarial/random graphs,
all-pairs answers in repeated and reversed query orders matched independent
Bellman-Ford at one and six workers. Tiny seeds 19, 20 and reserved 21 matched
the reference and Bellman-Ford at all four worker counts. The source was frozen
before seed 21 was generated. Published sample/full hashes matched at one, two
and six workers; the offline, read-only, unprivileged Docker run also matched.

Five interleaved baseline/candidate pairs used the same Victus CPU environment
and six workers. Time resolution was 0.01 seconds.

| Input | Baseline median time | Candidate median time | Baseline peak RSS | Candidate peak RSS |
| --- | ---: | ---: | ---: | ---: |
| Sample | 0.06 s | 0.05 s | 38,880 KiB | 30,948 KiB |
| Full | 0.95 s | 0.96 s | 294,296 KiB | 212,556 KiB |

Full peak RSS fell about 27.8%; the median time rose about 1.1%. The simple
nonblank-code-line count rose from 244 to 284, not the board's source-size
measure. The rough K-ratio proxy was 1.037765, including that penalty. The fixed
local gate required peak RSS at most 78% of baseline, speedup at least 0.90 and
proxy at least 1.035. It passed. The combined measurement does not separate the
benefit of earlier buffer release from compact distance storage, and does not
establish an official score.

## Compact distances and early input release — official observation

Commit `e0378c5` ran as `run-4aacdafae970` from 22:49:53 to 22:49:54 UTC
on 2026-10-30. Check, build, correctness, timeout and sandbox gates all passed.
The feed recorded T = 0.790007 s, M = 217,931,776 bytes, S = 292, D = 1,
K = 9.542026 and 48.982031 machine points. D improved by 0.337241 points
from 48.644790, but missed the predeclared 49.3-point threshold by 0.317969.
The score-target hypothesis failed; the upload, runtime and correctness checks
did not. The valid new best is retained without a timing-only retry.

Against the prior best run, official peak memory fell about 27.6%
(301,199,360 to 217,931,776 bytes), while runtime rose about 5.6%
(0.748345 to 0.790007 s). S increased from 252 to 292. The local full-input
median had risen only about 1.1%; the local proxy was not an official score
prediction. These combined measurements do not isolate the effects of compact
distance storage and earlier input release.

At 22:50:46 UTC, the board displayed 51.884 overall, rank 35. A, B and C
had unchanged source and lower measurements in this dispatch; their earlier
bests remained on the board. These are live machine results, not final judging,
an award or payment.

## Current phase diagnostic and 32-bit CSR indexes

A private, boundary-instrumented copy of the accepted compact-distance version
measured graph preparation at a median 0.432 s and the parallel worker interval
at 0.530 s on the full input. Forward and reverse edge-array fills each took
about 0.10 s. Worker durations overlap and are not added together. In five
interleaved full pairs, whole-process medians were 0.950 s without instrumentation
and 0.968 s with it, a 1.9% increase. Every output matched the published hash.
These measurements identified construction and graph access as costs to examine.

The candidate narrows CSR offsets and fill cursors from 64 to 32 bits. The
specified full edge count is 5,000,000; a checked conversion rejects counts
that do not fit before prefix sums are built. Edge ordering within each row,
search policy, distance storage and 64-bit path arithmetic are unchanged.

An independent adjacency-multiset oracle checked both graph directions, including
empty rows, duplicates and self-loops. The existing queue and distance properties
also passed. Seven host examples ran at one, two, six and twelve workers. On
109 adversarial/random graphs, repeated and reversed all-pairs queries matched
Bellman-Ford at one and six workers, including concentrated in/out-degree cases.
Tiny seeds 22, 23 and reserved 24 matched the reference and Bellman-Ford at all
four worker counts; source was frozen before seed 24 was generated. Sample/full
published hashes matched at one, two and six workers. The offline, read-only,
unprivileged Docker run also matched.

Five interleaved baseline/candidate pairs used six workers on the same Victus
CPU environment. Time resolution was 0.01 seconds.

| Input | Baseline median time | Candidate median time | Baseline peak RSS | Candidate peak RSS |
| --- | ---: | ---: | ---: | ---: |
| Sample | 0.05 s | 0.05 s | 30,764 KiB | 28,716 KiB |
| Full | 0.98 s | 0.92 s | 212,736 KiB | 192,688 KiB |

The full median fell about 6.1%, for a 1.0652x speedup, and peak RSS fell about
9.4%. Simple nonblank code lines rose from 284 to 285; this is not the board's
source-size measure. The rough K-ratio proxy was 1.052174. The fixed local gate
required speedup at least 1.05, peak RSS at most 95% of baseline and proxy at
least 1.03. It passed. The local proxy is not an official score.

## 32-bit CSR indexes — official observation

Commit `2f5cdaf` ran as `run-742c0a57da2c` from 23:06:09 to 23:06:11 UTC
on 2026-10-30. Check, build, correctness, timeout and sandbox gates all passed.
The feed recorded T = 0.762079 s, M = 197,136,384 bytes, S = 293, D = 1,
K = 9.907045 and 49.797206 machine points. D improved by 0.815175 points
from 48.982031 and passed the predeclared 49.6-point threshold.

Against the prior best run, official runtime fell about 3.5%
(0.790007 to 0.762079 s) and peak memory fell about 9.5%
(217,931,776 to 197,136,384 bytes). S increased from 292 to 293.
The local median time had fallen about 6.1%; local timings and the proxy remain
separate from the official measurements.

At 23:07:03 UTC, the board displayed 52.170 overall, rank 34. A, B and C
had unchanged source and lower measurements in this dispatch; their earlier
bests remained on the board. These are live machine results, not final judging,
an award or payment.

## Rejected local candidates: parallel build and grammar parser

Two later candidates were tested independently against the accepted CSR32
version. The first built forward and reverse CSR arrays concurrently over
immutable triples. The second consumed the exact single SP/LF delimiter after
each number, using the specification's required trailing newline. It retained
64-bit accumulation; inputs outside that grammar were not claimed supported.
The two changes were not combined.

Both passed the host examples, independent adjacency/distance/queue checks,
109 adversarial/random all-pairs graphs, published output hashes and offline
Docker execution. The parallel candidate used tiny seeds 25, 26 and reserved
27; the parser used 28, 29 and reserved 30. Source was frozen before each
reserved seed was generated. The parser also matched the standard decimal
parser across widths, zero, maximum 32/64-bit values, SP/LF boundaries and EOF.

Each candidate had its own five-pair interleaved full-input batch with six
workers. The baseline source was the same, but the batches were separate.

| Candidate | Baseline median | Candidate median | Baseline peak RSS | Candidate peak RSS | Speedup | Rough K proxy |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Parallel CSR build | 0.89 s | 0.86 s | 192,164 KiB | 185,508 KiB | 1.034884x | 1.023952 |
| Single-delimiter parser | 0.91 s | 0.88 s | 192,416 KiB | 192,448 KiB | 1.034091x | 1.016869 |

The parallel candidate missed its fixed speedup threshold of 1.07 and proxy
threshold of 1.025. The parser missed its thresholds of 1.04 and 1.02. Both
met their memory limits and produced correct outputs, but neither passed its
full local acceptance gate. No official push was made for either candidate.
The accepted CSR32 source was restored, and the rejected variants were kept
privately with their measurements. These local failures do not change the
recorded official best of 49.797206.

## Queue operation counts and rejected local candidates

A private instrumented copy of the accepted CSR32 source counted operations
across all six workers and both search directions. On the full input it recorded
7,158,380 expanded nodes, 21,074,642 examined edges, 8,693,796 relaxations and
451,627 stale queue items. The heap made 22,382,370 peek calls, including calls
from pop, and redistributed 32,688,342 entries in 1,822,219 batches. Counter
identities passed, repeated sample totals matched, and every answer hash matched
the published value. One full baseline/instrumented pair took 0.915/0.924 s,
about 1.0% longer with counters. This single pair describes instrumentation
overhead; operation counts do not measure where execution time is spent.

Two candidates were then tested separately against the unchanged CSR32 source.
Ready-minimum reuse cached each direction's minimum and consumed its prepared
queue item. Packed queue entries stored common keys and node IDs in one u64,
with a checked index into exact wide-key records when needed. Neither change
was combined with the other or with the earlier rejected candidates.

Both passed seven host examples at one, two, six and twelve workers, independent
queue/distance/CSR properties, published hashes and offline Docker execution.
Ready-minimum reuse matched Bellman-Ford on 111 adversarial/random graphs and
tiny seeds 31, 32 and reserved 33. Packed entries passed 110 graphs and seeds
34, 35 and reserved 36, plus a multiset test covering key/node limits, duplicate
keys and resets. Each source was frozen before its reserved seed was generated.

Each candidate used a separate five-pair interleaved full-input batch with six
workers and 0.01-second timing resolution.

| Candidate | Baseline median | Candidate median | Baseline peak RSS | Candidate peak RSS | Speedup | Rough K proxy |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Ready-minimum reuse | 0.90 s | 0.90 s | 191,800 KiB | 192,668 KiB | 1.000000x | 0.994957 |
| Packed queue entries | 0.93 s | 0.97 s | 192,540 KiB | 191,300 KiB | 0.958763x | 0.970507 |

Ready-minimum reuse missed its fixed speedup threshold of 1.05 and proxy
threshold of 1.02. Packed entries missed its thresholds of 1.05 and 1.025,
and its memory reduction of about 0.6% fell short of the required 3%. Both
candidates were rejected locally and retained privately with their measurements.
Neither was pushed to the official board. The accepted source was restored;
its recorded official best remains 49.797206. A time profile is needed before
using these counts to justify another optimization.

## CPU samples and popped-vertex meeting checks

Two private software profiles of the accepted CSR32 source collected 671 and
673 user-space CPU samples at 199 Hz. The binary used the same optimized build
flags plus debug information. The installed profiler could record addresses
but could not resolve ELF symbols or unwind DWARF stacks. Offline resolution
used the exact binary's executable-segment mapping and debug information;
the original recordings were reused without another workload run.

Distance lookup accounted for 323/671 and 373/673 exclusive sample locations
(48.1% and 55.4%). Search expansion accounted for 231/671 and 194/673
(34.4% and 28.8%), concentrated around adjacency access. One and two samples
remained outside the resolved application. These are CPU sample shares, not
wall-time phase shares or direct cache-miss measurements. Recorded elapsed
times were 0.952 and 0.897 s against one unprofiled control at 0.918 s; that
small set does not establish a stable profiling-overhead estimate. Outputs
matched the published hashes throughout.

The candidate removes the opposite-distance lookup and meeting-bound update
from each successful neighbor relaxation. It retains the check at every popped
vertex, valid-minimum expansion in alternating directions, and the existing
sum-of-minima stopping condition. Queues, distance storage and graph construction
are unchanged.

Six property tests passed, including all 19,683 three-vertex directed graphs
whose edges are absent, zero-weight or unit-weight, with all nine source/target
pairs checked against Bellman-Ford. This includes self-loops. Seven host examples
passed at one, two, six and twelve workers. Another 113 adversarial/random graphs
used repeated and reversed all-pairs queries. Tiny seeds 37, 38 and reserved 39
matched the reference and independent oracle at all four worker counts; source
was frozen before seed 39 was generated. Published sample/full hashes and the
offline, read-only, unprivileged Docker run also passed.

Five interleaved full-input pairs with six workers gave a baseline median of
0.93 s and candidate median of 0.84 s: a 1.107143x speedup, or about 9.7% less
elapsed time. Peak RSS was 192,532 versus 192,320 KiB. Timing resolution was
0.01 seconds. The rough K-ratio proxy was 1.052440; simple nonblank source lines
fell from 285 to 281, a different measure from the official source-size metric.
The fixed gate required speedup at least 1.07, RSS at most 1.03x baseline and
proxy at least 1.035. All three passed. These are local measurements; official
results must be recorded separately after submission.

The sample-input medians were 0.04 s for the baseline and 0.05 s for the
candidate. This small-input regression is one timing-resolution step; the
predeclared efficiency gate used the full input, where the gain above was measured.

## Popped-vertex meeting checks — official observation

Commit `b94f7ff` ran as `run-e3d5f818fda3` from 23:52:40 to 23:52:41 UTC
on 2026-10-30. Check, build, correctness, timeout and sandbox gates all passed.
The feed recorded T = 0.751131 s, M = 197,242,880 bytes, S = 289, D = 1,
K = 9.9985 and 49.996742 machine points. D improved by 0.199536 points
from 49.797206, but missed the fixed 50.8-point target by 0.803258.
The threshold was not lowered. This is a valid new best and a failed score
hypothesis, with no execution failure or same-code retry.

Against the prior best, official runtime fell about 1.4%
(0.762079 to 0.751131 s), memory rose about 0.05%
(197,136,384 to 197,242,880 bytes), and S fell from 293 to 289.
The local full median had fallen about 9.7%; that magnitude did not carry
over to this official run. Different inputs and environments may contribute,
but these observations alone do not identify the cause.

At 23:53:09 UTC, the board displayed 52.240 overall, rank 35, against a leader
at 77.785. A, B and C had unchanged source and lower measurements in this
dispatch; their prior bests remained. The valid D improvement is retained.
These are live machine results, not final judging, an award or payment.

## Post-change profile and rejected paired-distance layout

A private profile of the accepted popped-vertex version collected 594 and
599 user-space CPU samples. Distance lookup accounted for 273/594 and 296/599
(46.0% and 49.4%); search expansion accounted for 187 samples in each recording.
One sample in each recording remained outside the resolved application. Exact
ELF/debug resolution reused the captured addresses, and all output hashes matched.
The guest exposed six cores and twelve logical CPUs with affinity 0–11; this does
not establish equivalence to the official physical machine. Profile/control
elapsed ratios were 0.921 and 0.885 against one control, so this small set is not
a stable estimate of instrumentation overhead or a new performance gain.

The next candidate placed both directional inline distances beside each other
in one per-worker array of two-element u32 pairs. It retained separate exact
wide-value maps, queues and touched-node lists, with no shared mutable storage
between workers. The search algorithm and meeting-bound policy were unchanged.

Six property tests passed, including direction-isolated values and resets and
all 19,683 three-vertex graphs. Host cases, 113 adversarial/random graphs,
tiny seeds 40, 41 and reserved 42, published hashes and offline Docker execution
also passed. Source was frozen before the reserved seed was generated.

In five interleaved full-input pairs with six workers, baseline/candidate medians
were 0.84/0.85 s and peak RSS was 192,324/192,160 KiB. Speedup was 0.988235x;
the rough K-ratio proxy was 0.993740. Sample medians were 0.04/0.05 s, measured
at 0.01-second resolution. The candidate missed its fixed full-input speedup
threshold of 1.10 and proxy threshold of 1.035; it met the memory limit.
The paired layout was rejected locally and was never pushed. The accepted
popped-vertex source and official best of 49.996742 were retained.

## Rejected smaller-frontier direction selection

A private candidate expanded the direction with fewer queued radix entries and alternated equal-size ties. It kept exact valid minima, the popped-vertex meeting bound, and the sum-of-minima stopping test. The candidate passed six property tests, including all 19,683 three-vertex directed graphs with absent/zero/unit edges; 117 adversarial and random all-pairs cases; the host cases; tiny seeds 43, 44 and the reserved 45; and the published sample/full hashes. The read-only, unprivileged container also returned the expected full output.

Five interleaved local full-data pairs gave a median of 0.86 s for the accepted source and 0.85 s for the candidate. Peak RSS was 192,576 and 191,516 KiB. The 1.0118x speedup and 1.0048 rough coefficient ratio missed the fixed 1.12x and 1.05 gates. These are local measurements, not official scores.

Private counter copies reduced expanded vertices from 7,158,669 to 7,123,891 and examined edges from 21,075,641 to 20,972,317: about 0.49% fewer in each case. Counter outputs matched the published hash; their counts are not timing attribution. The modest work reduction did not establish the required efficiency gain. The candidate was not submitted, and the accepted source was restored.

## Rejected inlined distance getter

The accepted binary contained four direct calls to `Distances::get`, including one in the neighbour-processing loop. A private candidate marked the common getter `inline(always)` and moved the exact wide-map read to a cold, non-inlined helper. Disassembly confirmed that direct getter calls disappeared. Compact storage, bounds checks, search order and all arithmetic were unchanged.

The candidate passed the six property tests, 19,683 exhaustive three-vertex graphs, 117 adversarial/random cases, host cases, tiny seeds 46/47 and reserved 48, published sample/full hashes, and the unprivileged read-only container check. Five interleaved full pairs gave median times of 0.84 s for both versions. Peak RSS was 192,220 KiB for the baseline and 192,448 KiB for the candidate; the rough coefficient ratio was 0.9966. This missed the fixed 1.12x speedup and 1.05 coefficient gates.

Removing the calls did not produce the required whole-process gain. The earlier CPU-sample attribution identifies where time was sampled, not how much call overhead costs. These results do not establish cache-miss or memory-bandwidth dominance. The candidate was not submitted, and the accepted source was restored.


## BFS vertex numbering: two rejected storage designs

The first candidate built a temporary forward CSR graph and visited every vertex
in breadth-first order, continuing from the next unvisited vertex for disconnected
components. A bijection remapped both endpoints of every edge and query before
rebuilding the forward and reverse graphs. Weights, parallel edges, query order
and the search algorithm were unchanged. All preprocessing was included in timing.

Both designs passed the host cases at one, two, six and twelve workers, the
exhaustive three-vertex test, 117 adversarial/random all-pairs cases, published
sample/full output hashes, and the offline unprivileged container check. The
first used tiny seeds 52/53 and reserved 54. The second used 55/56 and reserved
57, and matched an independent BFS permutation on 1,233 additional graphs.
Each reserved input was generated after the candidate source was frozen.

Five interleaved full-data pairs used a blocking child wait and a monotonic
parent timer. The fixed gates required at least 1.12x speedup, at most 1.03x
peak RSS and a rough coefficient ratio of at least 1.05.

| Storage design | Baseline/candidate median | Baseline/candidate peak RSS | Speedup | Rough coefficient ratio |
| --- | --- | --- | --- | --- |
| Separate BFS queue and inverse map | 0.829925 / 0.703614 s | 192,820 / 198,944 KiB | 1.179517x | 1.059445 |
| One queue with bitset visitation and in-place permutation inversion | 0.835382 / 0.735016 s | 192,588 / 199,208 KiB | 1.136549x | 1.030893 |

The first design missed only the memory gate. The second retained the same BFS
order and reduced the workspace during discovery, but whole-process peak RSS did
not fall; it missed both the memory and coefficient gates. Neither was submitted.
The coefficient uses a simple nonblank source-line proxy, not the official size
measurement. No cache-miss attribution was measured. The accepted source was
restored after each test.


## BFS numbering with earlier permutation release

This candidate keeps the first BFS design's traversal and separate queue/map.
After remapping the edge triples, it reads and remaps the queries, then releases
the permutation and query input buffer before building the final CSR graphs.
The source change addresses allocation lifetime. It leaves the search algorithm,
weights, worker limit and output ordering unchanged.

Eight private properties passed, including the 1,233-graph BFS permutation
comparison and all 19,683 three-vertex graphs checked against Bellman-Ford.
Host cases at one, two, six and twelve workers, 117 adversarial/random all-pairs
cases, tiny seeds 58/59 and reserved 60, published sample/full hashes, and the
unprivileged offline container check also passed. Seed 60 was generated only
after the source was frozen.

Five interleaved full-data pairs gave baseline/candidate medians of
0.849322/0.705578 s: 1.203724x speedup, or about 16.9% less elapsed time.
Peak RSS was 192,780/192,220 KiB. These whole-process measurements include
parsing, BFS numbering, query remapping and CSR construction. The nonblank
source-line proxy rose from 281 to 319; the rough coefficient ratio was 1.077094.
The unchanged gates of 1.12x speedup, at most 1.03x RSS and at least 1.05
coefficient ratio all passed. This is a local result; official scoring uses
its own machine, inputs and source-size measurement.


## BFS early-release trial — official result and restoration

Commit `3f1dd28` finished official evaluation on 2026-10-31 at 03:06:24 UTC.
D passed check, build, correctness, timeout and sandbox gates, with
T = 0.756991 s, M = 196,579,328 bytes, S = 327, D = 1, K = 9.783461
and 49.524627 machine points. This missed the fixed 51.6-point target and
was below the previous best of 50.424918. The local speedup did not carry
over to this official trial. The measurements do not isolate the effects
of machine, input or run-to-run variation.

At 03:07:33 UTC, the board retained D's 50.425 best and showed 52.537 overall
at rank 38. A reached 57.929313 on unchanged source; B and C also reran without
source changes and retained their earlier bests. The small overall increase
therefore does not demonstrate a gain from the D candidate.

The BFS candidate was rejected after the official trial. The D source and
README were restored byte-for-byte from `0f25f4a`; this log retains the failed
trial. A corrective push keeps the final recorded source on the previously
accepted implementation. It is not another attempt to improve the score by
rerunning unchanged code. These are live machine results, not an award or payment.
