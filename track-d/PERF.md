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
