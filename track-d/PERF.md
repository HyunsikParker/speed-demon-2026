# Track D: performance log

## Live-board history

The first candidate is a Rust CSR implementation with epoch-tagged distance
arrays and, by default, up to six independent query workers.

| Run | Commit | Result | T (s) | M (bytes) | K | Machine points |
|---|---|---|---:|---:|---:|---:|
| `run-d2234e76683a` | `06d5a2c` | Correct | 1.689332 | 222,765,056 | 7.144156 | 42.697548 |
| `run-f0f2ac9f797d` | `8a1a566` | Correct, unchanged D source | 1.725366 | 222,875,648 | 7.068459 | 42.466237 |
| `run-44b5bb312166` | `0a3e541` | Correct, unchanged D source | 1.701160 | 222,834,688 | 7.118832 | 42.620437 |

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
