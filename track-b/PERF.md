# Track B: performance and validation

Snapshot: 2026-10-31. This report describes the accepted source in repository
commit `72deb7d7d2abc47fa32758336a67fcacaa8c3b49`.

## Recorded official result

Official run `run-a9ce34cd1404` on that commit finished at 07:59:24 UTC with
all five gates passing and **55.479075 machine points**.

| Official axis | Value |
|---|---:|
| T: process time | 0.918916 s |
| M: cgroup memory peak | 59,441,152 bytes |
| S: source size | 409 |
| D: dependency axis | 1 |
| K: coefficient | 12.870087 |

The score improved on the previous best of 55.036276 by 0.442799 points. It missed
the preregistered 56.5-point target; that threshold failure did not invalidate the
correct submission or erase the improvement. The source was retained. See
[MEASUREMENTS.json](../MEASUREMENTS.json) for the recorded own-track values.
These are live machine scores, not final judging or an award.

## Where the work and storage changed

The [implementation notes](README.md) cover tokenization, Boolean semantics and
shard ownership. Queries supply the dictionary: the program builds no index for
words that cannot affect an answer. Up to six workers read disjoint document
ranges through separate file handles.

Boolean evaluation uses blocks of 512 documents. A term receives a bitmap row
only when it occurs in the block; absent rows represent zero. Queries fold the
eight-word masks strictly left to right, with binary `NOT` interpreted as set
difference. A conservative presence check skips impossible results without
assuming that the presence of a subtracted term makes the result empty.

Phrase queries use a shared token-ID Aho–Corasick automaton. Matching happens
while each document is read, so positional posting lists are unnecessary.
Unknown terms reset the automaton, preserving gaps; document boundaries reset it
too. Repeated matches of one pattern in one document contribute one document ID.

The accepted dictionary stores eight prefix bytes, length and term ID in each
fixed-size slot. Longer keys also require complete byte equality. Inline query
records use the grammar's bounded term/operator counts. All structures are built
from the supplied input during execution, with no external crate or cached answer.

## Measured development steps

These are separate whole-program comparisons against the baseline of each step;
they must not be multiplied into a claimed combined speedup.

| Change | Local full-input baseline → candidate | Baseline → candidate peak RSS | Official result after the change |
|---|---:|---:|---:|
| Six document shards and bounded summaries | 4.16 → 1.21 s | 369,928 → 437,364 KiB | 46.165807 points |
| Streaming phrase automaton | 1.22 → 1.13 s | 437,568 → 185,296 KiB | 49.643708 points |
| Active Boolean bitmap rows | 1.121360152 → 0.954795051 s | 185,356 → 61,684 KiB | 55.036276 points |
| Compact dictionary and query records | 0.953204 → 0.813439 s | 61,620 → 57,340 KiB | 55.479075 points |

The first two rows use medians of three interleaved native pairs. The last two
use middle-three means from five paired pinned-image runs with the published
child timer. Container startup is excluded from those image times. All comparisons used
the same Intel Core i5-12500H/Ubuntu 24.04 WSL2 development machine. Sharded
implementations used six workers; the first baseline was single-threaded. Input
reading, construction, query evaluation and output are included.
RSS is peak process RSS from native runs, not official cgroup memory.peak.

The compact dictionary and query records had each missed their isolated adoption
gates. Their separately registered combination passed the unchanged 1.15× speed,
1.05 maximum RSS ratio and 1.06× coefficient-proxy gates. Its local source count
was 401, compared with official S = 409. Local ratios remain estimates rather than
predictions of the host score.

## Correctness checks

The accepted compact combination passed seven private tests and 18 additional
case/seed groups. These cover prefix collisions and probe wraparound, missing
terms, high term-ID bits, Boolean order and lengths, every byte-shard boundary,
and bounded result summaries. Tiny seeds 100 and 101 passed before reserved 102.

The wider validation record includes overlapping and suffix-related phrases,
duplicate queries, unknown-token resets, empty documents, long documents crossing
several ranges and ID sums exceeding 32 bits. Per-document predicates provide an
independent oracle alongside the host reference. Published sample/full outputs
matched exactly. The full seed-1 `results.txt` SHA-256 is:

`f86973c210228141bcb6f1f1b10188ad2a9b65f71e7cd7cf4590b38de4e80927`

Use the [repository checker](../README.md#build-and-check) with the corresponding
official dataset and expected hash.

## Rejected follow-ups

A four-token interleaved lookup candidate preserved token order and exact key
checks, but pinned-image time rose from 0.799053696 to 0.827242352 s. Its K proxy
was 0.976494543. Both timing spreads and correctness checks passed; the efficiency
gates failed, so the candidate was not submitted.

A subsequent diagnostic measured query reuse and negative dictionary lookups
without changing lookup results. Every Boolean query was checked over all truth
assignments of its terms, and sample/full outputs matched their published hashes.

| Diagnostic | Sample | Full | Predeclared candidate threshold |
|---|---:|---:|---:|
| Canonical prefix-operator reuse | 1.4906% | 1.6620% | At least 30% on both |
| One-bit prefilter rejection, as a fraction of all lookups | 37.1451% | 19.2479% | At least 60% on both |

The diagnostic passed its exactness checks, but neither candidate met its
opportunity threshold. The filter only counted possible rejections; it never
skipped the original lookup. These counters do not establish a runtime gain,
and neither idea was implemented as a submission candidate.

## Build-time PGO trial (October 31)

A separate candidate kept the accepted Rust source byte-identical and changed
only its build. The allowed LLVM-tools image collected a profile from one
sample dataset generated inside the offline build, using the official generator
and seed 130. The matching `llvm-profdata` merged 64 instrumented functions;
the final runtime image contained only the optimized program on the usual
Debian base. The candidate built in 16.06 seconds and occupied 30,058,994 bytes.

The binaries extracted from both images produced the expected sample and full
outputs. Eighteen correctness groups covered the host cases, independent
per-document predicates and fresh seeds 131–133, with 133 reserved until the
end. The fixed image batch measured 0.796115425 seconds for the accepted build
and 0.801751279 seconds for PGO. Both timing spreads passed. Peak process RSS
was 57,316 versus 57,576 KiB; these are local process measurements, not the
scoring machine's cgroup peak.

The speed ratio was 0.992971 and the K proxy was 0.992371. The source-size proxy
included the extra Dockerfile instructions and excluded only the byte-identical
official generator. This candidate missed the predeclared 1.15 speed and 1.06 K
gates, so it was not submitted. The accepted build remains unchanged. This one
profile provides no basis for claiming that PGO cannot help another workload.

## Limits and record

Each worker still visits the Boolean queries at every block. Phrase-match lists
can grow with the number of matching documents. CPU profiles locate recurring
dictionary and bitmap work, but do not establish cache-miss rates or wall-time
shares. These constraints guide further measurement, not an assumed speedup.

The [complete experiment log below](#complete-experiment-log) preserves the previous log unchanged, including
all earlier failures, controls, timing repairs and fixed gates. Statements that a
result was pending belong to their original observation time.

Codex implemented the candidate and ran the checks.

## Complete experiment log

<details>
<summary>Open the full chronological record, including rejected trials and measurement repairs</summary>

# Track B: performance log

## Live-board history

The first candidate replaces the Python index with a standard-library Rust
query-term index.

| Run | Commit | Result | T (s) | M (bytes) | K | Machine points |
|---|---|---|---:|---:|---:|---:|
| `run-750270f3b3c2` | `bc07ea2` | Correct | 5.433148 | 387,010,560 | 4.136591 | 30.832131 |
| `run-27458ab5e1fd` | `82216f0` | Correct, unchanged B source | 5.915574 | 389,976,064 | 3.958284 | 29.875346 |
| `run-856be110b4b8` | `17838c7` | Correct, unchanged B source | 5.571093 | 386,613,248 | 4.085897 | 30.56437 |

The [official live board](https://speeddemon.dev/board/) reported this run on
2026-10-30, from 20:37:54 to 20:38:00 UTC. Check, build, correctness, timeout and
sandbox gates passed. The board reported S = 174 and D = 1. At 20:39:16 UTC,
all four tracks had valid scores; the overall display was 40.969, rank 40.
B was below C, so it was not included in the best-three geometric mean.
These are live machine points, not final judging or an award.

The next run accompanied D's bidirectional-search update with unchanged B
source. All five gates passed; S = 174 and D = 1 were unchanged. Its latest
29.875346 points were below the prior best 30.832131, which the page
retained. At 20:51:13 UTC the overall display was 42.042, rank 38. This is a
same-source control, not an algorithmic regression or improvement claim.

## Local validation — 2026-10-30

Machine: Intel Core i5-12500H, Ubuntu 24.04 under WSL2, Rust 1.99.0 and Python 3.12.
Both programs ran on the same otherwise idle development machine. Each timed
program read the documents and queries, built its index, evaluated every query
and wrote the complete output. GNU `time` recorded one wall-time and
peak-process-RSS observation per program and scale. These are not official
five-run T or cgroup `memory.peak` M values.

| Public seed 1 data | Python reference wall time | Rust wall time | Reference peak RSS (KiB) | Rust peak RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 1.95 s | 0.18 s | 309,828 | 30,280 |
| Full | 38.00 s | 4.47 s | 2,150,408 | 369,804 |

The full-data pair showed a wall-time speedup of about 8.50×. One pair does not
establish repeatability or predict the board coefficient. Local reference
Python 3.12 also differs from official Python 3.13.

Correctness checks passed:

- The official worked example and all six host edge-case tests, checked against
  literal expectations, the reference and an independent per-document oracle.
- A 100,000-document case with a document-ID sum exceeding 32 bits, repeated
  phrase matches and phrases longer than a document.
- Unindexed intervening tokens and duplicate queries, preserving exact token
  positions and one output line per query.
- Tiny seeds 10 and 11, then reserved seed 12 generated after those checks
  passed. Each contained every host-listed edge-case category and matched both
  independent implementations. The candidate source remained unchanged.
- Published sample/full seed 1 input hashes and exact output hashes.
- Full output from the offline-built image as uid 65534, with no network,
  read-only root, an 8 GB memory limit, no swap and a 1 GB tmpfs. The image was
  30,028,821 bytes. This check did not use the official scoring harness.

Full output SHA-256:
`f86973c210228141bcb6f1f1b10188ad2a9b65f71e7cd7cf4590b38de4e80927`.

Tested `src/main.rs` SHA-256:
`df003504c4cd0fb56d4914fa044589d39b96ceccadb4cbae75f26cb73170d0de`.

## Interpretation

The candidate indexes only terms present in the queries and retains positions
only for phrase terms. Boolean queries use sorted document-ID lists; phrase
queries start from the shortest positional list and check exact offsets in the
others. The tokenizer still counts unindexed tokens, so dropping unused terms
does not alter adjacency. No ablation was run: the measurements cover the
language change, compact lists, selective indexing and query algorithms together.
Codex implemented the candidate and ran the checks.

## Same-source observation with the C bitset push

Commit `17838c7` changed C; this track's source was unchanged. Run
`run-856be110b4b8` passed all five gates and recorded 30.56437
machine points. The board retained the prior best 30.832131.
At 21:04:13 UTC, the overall display was 44.759, rank 38.

## Document shards and bounded summaries — 2026-10-30

A private phase-timing run of the accepted single-threaded code spent
3.576519 s building the index and 0.662760 s on queries and output. The new
implementation divides the input into up to six contiguous document ranges.
Each worker builds an independent index and answers all queries for its range.
It returns each query's count, ID sum and ten smallest matching IDs. Combining
those summaries preserves Boolean set operations and document-local phrases;
no full index merge or in-memory corpus copy is needed.

The first prototype was correct and faster, but failed its memory gate. Three
full-data pairs had median times of 4.19 s for the accepted code and 1.31 s for
the prototype, with peak process RSS of 369,928 and 847,928 KiB. A private
capacity probe found 444,833,280 bytes of posting and term-vector capacity,
plus 496,486,540 bytes in result summaries. Although each summary had at most
ten IDs, collecting `ids.into_iter().take(10)` reused the allocation of the
whole answer vector. The summary therefore retained far more capacity than
its length suggested. Copying the first ten IDs from a borrowed slice into a
fresh vector fixed that retention. No allocator setting was changed.

The corrected candidate was compared with the accepted single-threaded Rust
binary on the same otherwise idle Intel Core i5-12500H/Ubuntu 24.04 WSL2 machine.
Both used Rust 1.99.0 release optimization, LTO, one codegen unit and panic-abort.
Three pairs were interleaved baseline/candidate, candidate/baseline,
baseline/candidate. Each run read inputs, built indexes, answered queries and
wrote the complete output. The table gives median wall time and the maximum
process RSS among three runs, not official T or cgroup memory.peak M.

| Public seed 1 | Accepted wall time | Six-worker wall time | Accepted peak RSS (KiB) | Six-worker peak RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 0.18 s | 0.06 s | 30,200 | 39,348 |
| Full | 4.16 s | 1.21 s | 369,928 | 437,364 |

Full timings were 4.16, 4.20 and 4.16 s for the accepted binary, and 1.19, 1.24
and 1.21 s for the candidate: a 3.44× median ratio with an 18.2% RSS increase.
Sample RSS increased 30.3%. Parallel indexes and per-worker query summaries
still have a memory cost. A simple nonblank, noncomment source-line count grew
from 166 to 238; the board's normalized S may differ. No dependency was added.

Correctness checks passed:

- Every possible byte split in a fixture with UTF-8, empty documents and
  multi-digit IDs produced the same counts, sums and first ten IDs. A separate
  1,000-match case checked that a returned ten-ID summary retained no more than
  ten elements of vector capacity.
- The host worked example, all six host edge-case methods, 64-bit sums,
  repeated phrases, unindexed-token positions, a long document crossing several
  ranges and an empty corpus matched the host reference and an independent
  per-document predicate oracle. Worker settings 1, 2, 6 and 12 were checked;
  the last is clamped to six.
- Tiny seeds 28 and 29 passed before reserved seed 30 was generated. Each passed
  at one, two and six workers, included every host-listed edge category, and
  matched both independent implementations. Source bytes stayed frozen.
- All timed sample/full outputs matched the published hashes. The offline-built
  image also produced the full expected output as uid 65534, with a read-only
  root, no network, six CPUs, 8 GB memory without swap and a 1 GB tmpfs. Its size
  was 30,047,563 bytes. This was not the official scoring harness.

Tested `src/main.rs` SHA-256:
`1c09f4dd9e5928b9b182be994eff3edd281fe09bfee768a7c1f6e58f213f1696`.

Codex implemented and validated this change. Official live-board run
`run-a67cb4dd29a3`, commit `57f452c`, finished at 21:35:15 UTC with all five
gates passing. It recorded T = 1.1123 s, M = 460,935,168 bytes, S = 246,
D = 1, K = 8.381392 and 46.165807 machine points. This exceeds the prior B
best of 30.832131 by 15.333676 points, calculated from the two official values.
B now contributes to the team's best three tracks with A and D.

At 21:36:09 UTC, the board displayed overall 47.174 and rank 36. These are
live machine results, not the final judged total or an award.


## Streaming phrase matching — 2026-10-30

A private diagnostic of the accepted six-worker implementation measured
0.925–1.025 s building each shard's index and 0.160–0.174 s answering its queries.
Across the six indexes, position-vector capacity occupied 289,859,008 bytes of
444,833,280 bytes in posting and term metadata allocations. These are capacity
counts, not complete process memory measurements; simultaneous per-worker times
must not be added to obtain wall time.

The new implementation builds a token-ID Aho–Corasick automaton from the actual
phrase queries. Workers find phrase matches while reading documents, retaining
matching document IDs instead of every term occurrence and position. Boolean
queries keep their sorted-list operations. Phrase-only terms no longer need
Boolean posting lists. The six document shards and bounded result summaries
are unchanged. No external crate or ready-made search engine was added.

The accepted and candidate binaries used Rust 1.99.0 with release optimization,
LTO, one codegen unit and panic-abort on the same Intel Core i5-12500H/Ubuntu
24.04 WSL2 machine. Three pairs were interleaved baseline/candidate,
candidate/baseline, baseline/candidate. Each run included query parsing,
automaton or index construction, document reading, query evaluation and output.

| Public seed 1 | Accepted median wall | Candidate median wall | Accepted max RSS (KiB) | Candidate max RSS (KiB) |
|---|---:|---:|---:|---:|
| Sample | 0.05 s | 0.05 s | 41,100 | 21,076 |
| Full | 1.22 s | 1.13 s | 437,568 | 185,296 |

Full timings were 1.19, 1.22 and 1.22 s for the baseline, and 1.15, 1.13 and
1.12 s for the candidate. Median wall time fell 7.4%, and maximum process RSS
fell 57.7%. GNU time's process RSS is not official cgroup memory.peak. A simple
nonblank, noncomment source-line count grew from 238 to 296; the board's
normalized S may differ. The measurements cover this implementation as a whole,
including removal of phrase-only Boolean postings; no ablation isolates each part.

Correctness checks included every byte boundary in the shard fixture, bounded
summary capacity, the host worked example and all six host edge-case methods.
An independent per-document predicate oracle also checked all binary token
sequences through length eight against overlapping and suffix-related phrases,
including repeated patterns, duplicate queries, unknown-token resets and
separate documents. Existing 64-bit sums, long-document and empty-corpus checks
passed at worker settings 1, 2, 6 and 12, with 12 clamped to six.

Tiny seeds 31 and 32 passed before reserved seed 33 was generated, with frozen
source and exact host/reference/oracle agreement at one, two and six workers.
Every timed sample/full output matched its published hash. The offline-built
30,054,392-byte image produced the exact full output as uid 65534, with a
read-only root, no network, six CPUs, 8 GB memory without swap and a 1 GB tmpfs.
This was not the official scoring harness.

Tested `src/main.rs` SHA-256:
`93b3c83be530f5b756710a185252290821655186148769cb209375b0086a2ea8`.

Codex implemented and validated this change. Official live-board run
`run-7caf434b1ee3`, commit `a3a4156`, finished at 21:49:27 UTC with all five
gates passing: T = 1.077159 s, M = 191,315,968 bytes, S = 304, D = 1,
K = 9.83726 and 49.643708 machine points. Compared with the prior B best,
points rose by 3.477901 and M fell 58.5% (calculated from the official values).
At 21:50:14 UTC, the overall display was 48.330, rank 35. These are live
machine results, not final judging or an award.

A post-push correctness check, with identical source, also confirmed the
published sample/full output hashes at one and two workers. The pre-push
sample/full timing checks used six workers; fixtures and tiny seeds had already
covered one, two and six. No code change or second push was made.

## Same-source observation with the D radix push

Commit `47a7c57` changed D; this track's source was unchanged. Run
`run-f9d697ef68a5` passed all five gates and recorded 47.584457
machine points. The board retained the prior best 49.643708. The slower
measurement came from unchanged source and is not evidence of a code change.
At 22:11:03 UTC, the overall display was 48.815, rank 34.

## Same-source observation with the A parallel-range push

Commit `014bdb7` changed A; this track's source was unchanged. Run
`run-175213d36c86` passed all five gates and recorded 49.314582
machine points. The board retained the prior best 49.643708.
At 22:23:46 UTC on 2026-10-30, the overall display was 51.600, rank 35.
These are live board results, not final judging or an award.

## Same-source observation with the A lazy-group push

Commit `86a95c8` changed A; this track's source was unchanged. Run
`run-e68eefc850e0` passed all five gates and recorded 48.143062
machine points. The board retained the earlier best 49.643708.
At 22:38:49 UTC on 2026-10-30, the overall display was 51.765, rank 35.
This rerun is not evidence of a new algorithm, final judging or an award.

## Same-source observation with the D compact-storage push

Commit `e0378c5` changed D; this track's source was unchanged. Run
`run-061d28326788` passed all five gates and recorded 48.462629
machine points. The board retained the earlier best 49.643708.
At 22:50:46 UTC on 2026-10-30, the overall display was 51.884, rank 35.
This rerun does not establish a new algorithm improvement, final judging or an award.

## Same-source observation with the D CSR-index push

Commit `2f5cdaf` changed D; this track's source was unchanged. Run
`run-703772e9645b` passed all five gates and recorded 49.127098
machine points. The board retained the earlier best 49.643708.
At 23:07:03 UTC on 2026-10-30, the overall display was 52.170, rank 34.
This rerun does not establish a new algorithm improvement, final judging or an award.

## Same-source observation with the popped-vertex meeting push

Commit `b94f7ff` changed D; this track's source was unchanged. Run
`run-6525f42a83b9` passed all five gates and recorded 49.38212
machine points. The board retained the earlier best 49.643708.
At 23:53:09 UTC on 2026-10-30, the overall display was 52.240, rank 35.
This rerun does not establish a new algorithm improvement, final judging or an award.


## Rejected local experiment: query-word hashing

A hash-only candidate on 2026-10-31 replaced the query-word map's default
hasher with FNV-1a, retaining byte-key equality, collision resolution and all
search algorithms. Forced-collision tests, every shard boundary, host edge
cases, phrase overlaps, Unicode handling, seeds 34/35 and reserved seed 36
passed. The published sample/full hashes and offline unprivileged container
output also matched.

Five interleaved full pairs measured median wall time of 1.10 s for the
accepted code and 1.07 s for the candidate, a 1.028x ratio. Maximum process RSS
was 185,264 and 185,356 KiB. The local coefficient-ratio proxy was
1.006818, below the fixed 1.20 gate; speedup also missed the 1.50 gate.
These process-RSS and source-line proxies are not official scoring values.
The candidate was not pushed. The accepted source was restored, and hash
constant or seed sweeps are not justified by this result.


## Current-source CPU diagnostic

Two private software-CPU profiles of the accepted source on 2026-10-31
contained 1,274 and 1,284 samples. Exact ELF inline/source mapping placed
341/340 samples at the query-word lookup, 207/210 at posting-list append, and
94/100 at the preceding duplicate-document check. The dictionary lookup
therefore accounts for 26.8%/26.5% of samples; append and duplicate checking
together account for 23.6%/24.1%. These are CPU sample locations, not wall-time
phase shares or proof of allocation or cache-miss costs. Other DSOs account
for 18.3%/19.2% and remain unresolved.

The source and published full output were unchanged. The initial analysis
misread the report's abbreviated `1K` sample header. Exact event rows from the
saved profiles corrected the count and matched the sum of per-location counts;
the workload was not rerun. Static inline frames do not reconstruct a runtime
callgraph, and the single unprofiled control does not isolate profiler overhead.


## Adaptive Boolean postings — local rejection, 2026-10-31

This candidate kept sparse document lists as vectors and converted observed dense
lists to offset bitsets. It switched back when the document span grew too sparse.
Phrase matching, the token dictionary and the six document shards were unchanged.

All correctness checks passed, including representation changes, word boundaries,
large IDs, every three-term Boolean operator combination, shard boundaries, the
host cases, independent per-document predicates, seeds 37/38 and reserved seed 39.
Published sample/full hashes and the constrained offline container check passed.

Five interleaved full-data pairs on the same otherwise idle Victus machine had
median times of 1.12 s for the accepted source and 1.21 s for the candidate.
Maximum process RSS was 185,268 versus 164,624 KiB. Memory fell 11.1%, but runtime
increased 8.0%. The simple nonblank, noncomment line count rose from 296 to 424.
These are local measurements, not official T, cgroup memory.peak or normalized S.

The fixed gate required a speed ratio of at least 0.95, no RSS increase and a
combined time/memory/source-size coefficient ratio of at least 1.20. The measured
ratios were 0.92562, 0.88857 and 0.93340 respectively. The candidate failed and
was not pushed to the board. The accepted source was restored; no density
threshold sweep or lower acceptance gate was used. Codex implemented and tested
this candidate.

## Compact query-token trie — 2026-10-31

A private candidate built a compact trie from the query vocabulary and matched
words while scanning ASCII tokens. Child masks and contiguous child spans kept
the trie smaller than a dense transition table. The existing Boolean postings,
phrase automaton, shard boundaries and output logic were unchanged.

All 17 validation groups and five private tests passed, including every byte
split in the shard fixture, ASCII classification, prefixes, absent words,
generated vocabulary and the 32-byte token limit. Tiny seeds 40 and 41, then
reserved seed 42, matched the reference and independent per-document oracle.
The published sample/full hashes and constrained offline container output also
matched.

Five interleaved full-data pairs measured median wall times of 1.111188365 s
for the accepted code and 1.085468384 s for the trie: a 1.023695× speedup.
Peak process RSS rose from 185,248 to 190,888 KiB. The source-adjusted local
coefficient proxy was 0.975412; its line-count inputs were 296 and 363, not
an official S measurement. Timings used a blocking child wait under a separate
600-second outer limit.

The candidate missed the fixed speedup ≥ 1.30 and coefficient-proxy ≥ 1.10
gates. Its RSS ratio, 1.030446, was within the 1.05 limit. It was rejected
without a board push, and the accepted source was restored. This result does
not show that dictionary lookup is free; it shows that this trie did not reduce
the total cost enough to justify its size and memory use.

## Inline document stamp — 2026-10-31

A separate candidate kept each Boolean term's last-seen document ID plus one
in its metadata. This removed the posting-vector tail read from duplicate
checks. The dictionary, tokenizer, phrase automaton, postings and query logic
were unchanged; the zero initial stamp could not match a valid document.

All 16 validation groups and both private boundary/summary tests passed, with
tiny seeds 43 and 44 followed by reserved seed 45. The full offline container
output also matched. Five interleaved full-data pairs measured 1.104265414 s
for the accepted code and 0.994869191 s for the candidate, a 1.109960× speedup.
Peak process RSS was 185,296 versus 185,308 KiB. The local coefficient proxy
was 1.051941, using source-line proxies of 296 and 299.

This missed the fixed 1.15× speed and 1.06 coefficient-proxy gates. The
standalone candidate was not pushed. Its measured reduction in execution time
is retained as evidence for a separately registered integration; the original
baseline and adoption gates remain in place.

## Exact word-lookup cache with the stamp — 2026-10-31

The next candidate combined the fixed document stamp with a 1,024-slot
per-worker lookup cache. Cache entries held the complete token bytes, length
and resolved term ID, including absent terms. Every hit required exact byte
equality; collisions fell back to the unchanged dictionary. Tokens longer
than 32 bytes also used the dictionary. Cache placement and capacity were
fixed before testing and were not tuned to a seed.

All 16 validation groups and four private tests passed, including forced
known/unknown collisions and repeated eviction. Tiny seeds 46 and 47, then
reserved seed 48, matched both independent implementations. Published hashes
and the full constrained offline container output matched as well.

Five interleaved full-data pairs measured 1.147780619 s for the accepted code
and 1.027011652 s for the combined candidate, a 1.117593× speedup. Peak process
RSS was 185,272 versus 185,484 KiB. The local coefficient proxy was
1.038880, using source-line proxies of 296 and 332.

The combined candidate missed the unchanged 1.15× speed and 1.06
coefficient-proxy gates, so it was not pushed. These measurements compare the
combination against the original accepted code. They do not isolate the cache's
contribution relative to the separate stamp experiment. The accepted source
was restored; no cache-capacity or placement sweep was run.

## Current-code phase timing — 2026-10-31

A private copy of accepted source `93b3c83b` added coarse monotonic timestamps
around preprocessing, each shard's indexing and query evaluation, major-index
release, join and output. The earlier phase probe used the superseded positional
index, so its phase shares did not describe the current streaming implementation.
No counters were added inside the token loop, and the submitted source stayed
unchanged.

One sample profile and two full profiles matched the published output hashes.
Two exact-source full controls took 1.139715 and 1.171590 s; the instrumented
runs took 1.200161 and 1.123394 s. Across the 12 full-data shard observations,
indexing occupied 87.03–89.05% of each shard's measured phase interval, query
and summary evaluation 10.66–12.69%, and explicit major-index release
0.22–0.44%. Preprocessing took 0.030–0.034 s and final output 0.008–0.012 s.

These overlapping worker durations cannot be added to obtain process wall
time. Logging and explicit drops may affect the timings; this diagnostic does
not establish an efficiency gain or identify allocator internals. It supports
keeping the index as the main target and testing a bounded reduction of redundant
query-result materialization. It gives no reason to replace the allocator solely
to accelerate teardown.

## Direct final-query summaries — 2026-10-31

This candidate combined the fixed document stamp with direct accumulation of
the final Boolean operation's count, ID sum and ten smallest IDs. Intermediate
Boolean operations still built sorted lists. Single-term and phrase queries
borrowed their existing lists instead of cloning them before summarization.
The set visitor retained the accepted unbalanced-intersection shortcut and
left-to-right operator order.

All 16 validation groups and four private tests passed, including independent
BTreeSet comparisons for empty, overlapping, unbalanced and high-ID lists.
Tiny seeds 49 and 50, then reserved seed 51, matched the reference and independent
oracle. Published hashes and full offline container output matched.

Five interleaved full-data pairs measured 1.105568737 s for the accepted code
and 1.025490100 s for the candidate, a 1.078088× speedup. Peak process RSS rose
from 185,216 to 188,976 KiB. The local coefficient proxy was 1.019748, using
source-line proxies of 296 and 325. The candidate missed the fixed 1.15× speed
and 1.06 coefficient-proxy gates and was not pushed. The original accepted
source was restored. This combined test does not isolate the query reduction's
effect from the stamp's effect.


## Bounded bitmap blocks — rejected, 2026-10-31

This candidate replaced full-lived posting lists with 512-document bitmaps.
Each worker evaluated queries after a block and retained only counts, ID sums
and the first ten matching IDs. The block size was fixed before measurement;
no size sweep was run. ASCII tokenization, phrase matching and six workers
were unchanged.

All 17 correctness groups passed, including every byte-shard boundary,
512-document rollover and partial blocks, the independent per-document oracle,
tiny seeds 61 and 62 followed by reserved seed 63, and published sample/full
hashes. Five interleaved full pairs used the pinned images and the published
child timer. Their middle-three means were 1.123471812 s for the accepted code
and 1.162477289 s for this candidate; both spread checks passed. Native peak
process RSS fell from 185,292 to 95,028 KiB. RSS is a proxy, not the official
cgroup memory peak. The source-adjusted coefficient proxy was 1.1110 times
the baseline, below the fixed 1.15 gate; the 0.5129 RSS ratio also missed its
0.40 gate. The candidate was not submitted.

An allocation audit found 8,564,352 bytes of bitmap rows for phrase-only terms
that were never written, plus 7,275,648 bytes of phrase bitmap rows across six
workers. These are allocation formulas from the frozen source and full query
metadata, not measured attribution of RSS. The source also reserved ten IDs
for every query/shard summary before any match. This supports testing active
bitmap rows, sparse phrase results and lazy summary allocation while keeping
the block size and acceptance gates fixed. Codex implemented and checked the
candidate.


## Active bitmap rows — 2026-10-31

The revised implementation allocates a block bitmap row only when a Boolean
term occurs. Phrase results use sparse, deduplicated document-ID lists, and a
query summary allocates its first-ten storage on the first match. The block
size remains 512 documents, with the same dictionary, phrase automaton and
six workers. This removes unused storage identified in the preceding audit.

The frozen candidate passed the same 17 correctness groups, using tiny seeds
64 and 65 followed by reserved seed 66. Published sample/full hashes matched
in every timed run. Five interleaved full pairs of the pinned images, measured
inside the container with the published child timer, gave middle-three means
of 1.121360152 s for the accepted code and 0.954795051 s for the candidate:
a 1.1745× ratio. Both middle-three spread checks passed.

Native peak process RSS was 185,356 KiB for the baseline and 61,684 KiB for the
candidate, a ratio of 0.3328. This is not canonical cgroup memory.peak. The
nonblank, noncomment source-line proxy grew from 296 to 319, with no dependency
change. Combining the image timing, RSS and source proxies gave a 1.3354×
coefficient ratio. It passed the original time, memory and coefficient gates;
the official box must still establish the score. Victus WSL timings and its
writable private timer-output mount do not reproduce the full official harness.

Tested `src/main.rs` SHA-256:
`be3f3095e8b1bd8311ecd05645364651bd00fd97de670e14edc2ebde9d987cd2`.

Codex implemented the revision and ran the checks. No official score gain is
claimed from these local measurements.


Official run `run-07e1054d8717` on commit `68ed36d4fcd88ac708773c298b926d313e504a8d`
finished at 2026-10-31 03:49:47 UTC with all five gates passing. It recorded
T = 0.987719 s, M = 64,991,232 bytes, S = 327, D = 1, K = 12.610303 and
55.036276 machine points. That is 5.392568 points above the prior B best.
At 03:49:56 UTC, the board displayed overall 55.449 and rank 38. A subsequent
root read confirmed all four tracks had finished on the exact commit. A, C and
D source bytes were unchanged and their previous best scores remained. These
are live machine results, not an award or payment. This result note is retained
locally; no extra push was made just to rerun unchanged code.


## Exact short-word lookup — rejected, 2026-10-31

A private candidate used a direct-address table for all 47,988 combinations of
one to three lowercase ASCII letters or digits. Length-separated base-36
positions preserved complete byte identity, including leading zeros. Entries
came from the current query file; longer tokens used the existing standard
hash map. The tokenizer, phrase automaton, active bitmap blocks and six workers
were unchanged.

Five private tests passed, including every short key, sparse and absent entries,
long-key lookup and every byte-shard boundary. All 18 correctness groups passed,
including tiny seeds 91 and 92 followed by reserved seed 93. Published outputs
matched in the fixed timed runs.

The pinned-image middle-three means were 0.959844 s for the accepted source
and 0.922765 s for the candidate, a speed ratio of 1.040183. Both spread checks
passed. Native peak RSS was 61,680 versus 61,688 KiB. The local source-adjusted
efficiency proxy was 0.996221, using 319 and 373 nonblank, noncomment lines.
The candidate missed the unchanged 1.15 speed and 1.06 efficiency gates. It was
not submitted, and the accepted source was restored.

## Token lookup census

A separate private copy of the accepted source counted query-dictionary hits
and misses by token length. A manually counted fixture passed. The full run
matched the published output hash, 250,000 documents and 36,276,657 tokens.
Of those lookups, 28,716,697 found a query term. There were 10,854,229 tokens
of one to three bytes: 29.92% of all lookups. Longer keys accounted for the
remaining 70.08%, including 17,963,796 hits and 7,458,632 misses.

These are work counts, not CPU or wall-time shares. They do not establish why
the short-word candidate improved elapsed time by the measured amount. No
instrumented timing was used as a benchmark, and no new official score is claimed.


## Query activation — local rejection, 2026-10-31

Two private diagnostics counted Boolean-query visits across 492 document blocks.
The accepted source visited 15,252,000 Boolean queries; its existing presence
check allowed 5,241,641 to proceed to exact bitmap evaluation. A condition based
on the first term and every OR term would skip 45.77% of visits, below the fixed
50% threshold for an implementation trial.

A second necessary condition starts at the last AND term and includes only
later OR terms. After that AND, matches must belong to its term set; later NOT
operations only remove documents, while OR can add its own term set. With no
AND, the condition starts at the first term. This condition activated 7,615,848
query visits and skipped 50.07%. No query passing the original presence check
was omitted. Both diagnostics preserved the published output hash. These
counts did not establish a runtime improvement.

The candidate stored the second condition in a compact term-to-query index.
Each worker queued a query once when an anchor term first appeared in a block,
then used the unchanged exact evaluator. The dictionary, tokenizer, phrase
automaton, 512-document blocks and six workers were unchanged.

The first private-test build failed because a test predicate mixed integer
types; the release build passed, and no workload ran. Explicit test-only type
annotations fixed the error without changing the candidate. The repaired trial
passed three private tests and all 18 correctness groups, including exhaustive
mixed operators, repeated and missing terms, duplicate activation, block and
shard boundaries, and tiny seeds 94, 95 and reserved 96.

Five fixed pinned-image pairs gave middle-three means of 0.972708 s for the
accepted source and 1.137978 s for the candidate, a speed ratio of 0.854769.
Both spread checks passed. Native peak RSS was 61,744 versus 62,324 KiB. The
source-adjusted local efficiency proxy was 0.892685, using line-count proxies
of 319 and 398. The candidate missed the unchanged speed and efficiency gates.

The accepted source was restored without an official submission. Fewer query
visits did not improve total runtime in this trial. It did not separately
measure activation overhead, access order or cache effects, so none is claimed
as the measured cause. The original compiler failure remains recorded apart
from this completed performance result.


## Inline query records — local rejection, 2026-10-31

A query-only diagnostic measured the accepted representation on the published
50,000-query input. Each query had a 64-byte header with separate term and
operator vectors. Including allocated capacities, query storage occupied
6,225,320 bytes across the outer vector and 79,709 inner allocations.

The candidate put up to eight checked unsigned 32-bit term IDs and five
operators in each 48-byte record, alongside the actual length, phrase flag and
existing pattern ID. At the same outer-vector capacity, this required
3,145,728 bytes: 49.47% less allocated query storage. That estimate excludes
allocator overhead and does not predict process RSS or runtime. The dictionary,
tokenizer, sequential query visits, bitmap evaluation, phrase automaton and
six workers stayed unchanged.

The first validation stopped because a private fixture contained literal
backslash-n sequences instead of line breaks. Correcting only that fixture
preserved the candidate bytes and all gates. Four private tests and all 19
correctness groups then passed, including every byte-shard boundary, bounded
summaries, all Boolean lengths and operator sequences over repeated terms,
missing terms, maximum-length phrases, high term-ID bits and tiny seeds 97,
98 and reserved 99. Published sample and full output hashes matched.

The first timing batch failed the baseline spread check. Its apparent speed
gain was not accepted. One separately registered measurement repair retained
the same sources and images, checked idle readiness before each phase and ran
two warmup pairs followed by five fixed pairs. It introduced no new holdout,
source change or parameter choice.

Both spread checks passed in that batch. Pinned-image middle-three means were
0.951543 s for the accepted source and 0.871392 s for the candidate, a speed
ratio of 1.091980. Native peak RSS was 61,616 versus 57,448 KiB, a ratio of
0.932355. With source-line proxies of 319 and 330, the local efficiency ratio
was 1.054345.

The candidate missed the unchanged 1.15 speed and 1.06 efficiency gates. The
accepted source was restored without an official submission. The fixture
error, unstable measurement batch and final performance result remain
separate records; the earlier, larger speed estimate was not selected.


## Compact dictionary and query records — local pass, 2026-10-31

This candidate combined the previously measured inline-prefix dictionary with
the fixed query records. Each isolated version had missed the adoption gates.
Their separate speed ratios were not added or multiplied to predict a result.

The dictionary retained its original dense term IDs, full-byte equality for
long keys and fixed probing layout. Query records held those IDs directly,
so freeing short-key byte buffers could not invalidate a query reference.
The tokenizer, phrase automaton, sequential query visits, active bitmap
blocks and six workers were unchanged.

Seven private tests passed, covering exact dictionary lookup, prefix collisions
and wraparound, missing terms, Boolean order and lengths, high term-ID bits,
every byte-shard boundary and bounded summaries. All 18 additional case and
seed groups passed, including tiny seeds 100 and 101 followed by reserved 102.
Published sample and full outputs matched exactly.

After idle-readiness checks, two pinned-image warmup pairs preceded five fixed
measurement pairs. Both middle-three spread checks passed. Mean times were
0.953204 s for the accepted source and 0.813439 s for the candidate, a speed
ratio of 1.171820. Native peak RSS was 61,620 versus 57,340 KiB, a ratio of
0.930542. The source-line proxies were 319 and 401, giving a local efficiency
ratio of 1.061158.

The candidate passed the unchanged 1.15 speed, 1.05 maximum RSS ratio and 1.06
efficiency gates. These are local measurements; an official score is pending.
The previous isolated failures remain part of the record.


## Official result: joint compact dictionary and queries (v26)

Commit `72deb7d7d2abc47fa32758336a67fcacaa8c3b49` was pushed once at
2026-10-31 07:58 UTC. Official run `run-a9ce34cd1404` finished at
07:59:24 UTC with all five gates passing. Track B scored **55.479075**,
up **0.442799** from the previous best of 55.036276. The board showed
57.388 overall and rank 40 when read at 08:02:18 UTC. These are score
observations, not an award or payment.

The official measurements were T=0.918916 s, M=59,441,152 bytes, S=409,
D=1, and K=12.870087. The measured source axis differs from the local
401-line proxy; local ratios were not predictions of the official score.

The fixed B >= 56.5 acceptance threshold was missed by 1.020925 points.
This is a scientific threshold failure, not an upload or execution fault.
The valid new best and its source are retained. A, C, and D source bytes
were unchanged; their latest reruns did not replace their recorded bests.
No unchanged retry, further retiming, parameter sweep, target relaxation,
or documentation-only push is justified by this result.


## Current-source profile and four-lane probe experiment (v27/v28)

Two process-scoped CPU profiles of accepted source `b678b95a` contained
844 and 848 accounted samples. The dictionary slot-load site held 144 and
141 samples (17.06% and 16.63%). These locate CPU work; they do not measure
cache misses, prove memory stalls, or give wall-time shares. Sample and full
outputs matched their published hashes. The accepted source was unchanged.

One fixed candidate gathered four tokens, loaded each unresolved probe
position, and resolved the lanes with the existing exact key checks. It
consumed results in token order so phrase boundaries and unknown-token
resets stayed intact. Table size, hash, key representation, bitmap block
size and worker count were unchanged. No batch-size sweep was run.

Nine private tests and 18 additional case/seed groups passed, including
reserved seed 105. The two warmup pairs and five measured image pairs used
the same pinned images and child timer as the baseline. Both spread checks
passed.

| Local metric | Accepted v26 | Four-lane v28 |
|---|---:|---:|
| Image middle-three mean | 0.799053696 s | 0.827242352 s |
| Native peak RSS | 57,280 KiB | 57,344 KiB |
| Source-line proxy | 401 | 418 |

Speedup was 0.965924549 and the source/RSS-adjusted K proxy was 0.976494543,
below the fixed 1.15 and 1.06 gates. RSS ratio 1.001117318 passed its 1.05
limit. The fixed four-lane candidate was rejected for efficiency. These
measurements do not isolate the source of its overhead. The accepted v26
source was restored; no official submission or unchanged retiming followed.

</details>
