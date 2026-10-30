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
