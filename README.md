# Bounded Accord — Speed Demon 2026

Four exact Rust implementations for [Speed Demon 2026](https://speeddemon.dev/).
Each track builds and runs independently. The programs use the Rust standard
library without external crates; the unchanged Python starter files remain as
reference material. The containers execute the Rust binaries.

## Implementations

| Track | Approach | Output | Details |
|---|---|---|---|
| A: Ingest | Parallel log ranges, exact endpoint interning, per-group latency sorting and ordered output | `summary.txt` | [Approach](track-a/README.md) · [Measurements](track-a/PERF.md) |
| B: Index | Query-derived dictionary, bounded Boolean bitmaps and a token-pattern automaton | `results.txt` | [Approach](track-b/README.md) · [Measurements](track-b/PERF.md) |
| C: Join | Read-only mapping, parallel quote-parity cuts and exact name/country membership | `aggregates.txt` | [Approach](track-c/README.md) · [Measurements](track-c/PERF.md) |
| D: Traverse | Forward/reverse CSR graphs, bidirectional Dijkstra and monotone radix queues | `distances.txt` | [Approach](track-d/README.md) · [Measurements](track-d/PERF.md) |

All indexes and aggregates are built from the supplied files during execution.
The programs do not recognise scoring inputs or ship cached answers. Each reads
its track's files from `/data`, writes exactly one file to `/out`, and exits.
The track READMEs describe boundary handling, exactness and remaining limits.

## Recorded live-board results

The last own-entry readback at 2026-10-31 08:02 UTC retained these best machine
scores. Each linked report gives its run ID, measured axes, accepted design and
validation limits.

| Track | Best machine points | Performance report |
|---|---:|---|
| A | 61.431366 | [Ingest](track-a/PERF.md) |
| B | 55.479075 | [Index](track-b/PERF.md) |
| C | 55.455380 | [Join](track-c/PERF.md) |
| D | 50.424918 | [Traverse](track-d/PERF.md) |

A and D's best runs accompanied changes to other tracks, with their own source
unchanged. Their score variation is not attributed to a new algorithm. The
[recorded score data](MEASUREMENTS.json) separates these controls from changed-source
runs. Final judging and awards are not established by the live board.

## Build and check

The Dockerfiles use digest-pinned images from the official allowlist and build
with `cargo build --release --offline`. Docker and Python 3 are needed for the
starter checker. From this repository's root, with a prepared Track A dataset:

```sh
python3 check.py a --data /absolute/path/to/a-data --expected EXPECTED_SHA256
```

Replace `EXPECTED_SHA256` with the published hash for that exact seed and scale,
or a hash independently obtained from the reference implementation. Use `b`,
`c` or `d` and the matching dataset for another track. Data generation and the
reference implementations are in the
[official specifications repository](https://github.com/hackathon-raptors/speed-demon-2026).

The checker builds without network access, runs as an unprivileged user with a
read-only root filesystem, and checks the output filename and hash. Its displayed
time includes container startup and is not the board's process-only timing.
The checker also tries to fetch the current image allowlist before building;
the solution itself needs no network.

For a native development run, enter a track directory and build offline. For
example, in `track-d`:

```sh
cargo build --release --offline
mkdir -p /absolute/path/to/output
./target/release/traverse /absolute/path/to/d-data /absolute/path/to/output
```

Each track README gives its executable name. Tracks A, B and D accept
`SD_WORKERS` and default to six workers. Track C uses six fixed workers for
quote-parity scanning and six for parsing; the two phases run in sequence.

## Measurement and validation

Each performance report starts with the accepted implementation and its evidence,
then retains the full experiment log in an expandable section. Historical entries
keep their original measurements, failures and qualifications. Local
comparisons include input parsing, construction, computation and output. Process
RSS and nonblank source-line counts used in local estimates are identified as
proxies for the board's memory and source-size measurements. The latest Track C
trial measures Docker cgroup memory directly; its report describes the
differences from the official harness.

Validation covers the published sample/full hashes, fresh generated inputs and
independent oracles. Additional checks cover collisions and shard boundaries in
A and B, quoted CSV records and exact membership in C, and zero-weight paths,
parallel arcs and distances above 32 bits in D. Rejected candidates are recorded
with their measured results; the working source retains the accepted versions.
Live-board results are practice measurements, not final judged rankings or awards.

## Attribution and licence

Codex developed the implementations and validation. Algorithm
references and applicable limitations are listed in each track README. The
organisers' starter files and this entry use the [MIT licence](LICENSE).
