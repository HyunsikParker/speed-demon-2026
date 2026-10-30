# Track A: performance log

## Live-board history

The first candidate replaces Python timestamp parsing and object-heavy groups
with a single-threaded Rust byte parser and compact integer latency vectors.
The [official live board](https://speeddemon.dev/board/) recorded the first run
on 2026-10-30, starting at 20:12:40 UTC and ending at 20:12:41 UTC.

| Run | Commit | Result | T (s) | M (bytes) | K | Machine points |
|---|---|---|---:|---:|---:|---:|
| `run-713a55a5de05` | `8a1a566` | Correct | 0.671122 | 37,187,584 | 9.188717 | 48.162744 |
| `run-6ab97c79674a` | `0a3e541` | Correct, unchanged A source | 0.671838 | 37,072,896 | 9.189495 | 48.164582 |

Check, build, correctness, timeout and sandbox gates all passed. The board
reported S = 94 and D = 1, and displayed rank 48 in Track A at 20:14:55 UTC.
These are live-board observations, not final five-run measurements. With only
A and D scored at that time, the team did not yet have an overall score.

The second run accompanied the first Track C implementation push. A's source
was unchanged; all five gates passed, with S = 94 and D = 1 again. It ran from
20:21:21 to 20:21:22 UTC. At 20:22:29 UTC the team had three scored tracks and
an overall 40.591, rank 40. The small A measurement change is a same-source
observation, not an optimization claim.

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
