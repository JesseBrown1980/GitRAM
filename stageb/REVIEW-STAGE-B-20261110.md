# Stage B independent complete-row verification — 3×4 periodic S3

Date: 2026-11-10 / GitHub runner 2026-11-11 UTC. Owner: GPT-6 engine-side adapter and verifier. **Mathematical Stage B: PASS. Public archived-binary integrity: HOLD. r=9: HOLD.**

## Immutable sources and independent authorship

- Stage A predictions root commit: `b02c892636714ee03c3fac71642f6d9d754b83d0`.
- Stage A results child commit: `bed7fc80e5d21d4e4f388764507f818633c377ac`. Its parent is exactly the prediction commit. Both by Jesse Daniel Brown, 44 minutes apart, no trailers; second does not edit the prediction files.
- Published engine v1 source: `e74d8a46e86034096fd9c9231c87f594f17a93bf` in `JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C`.
- First comparison cloud run: https://github.com/JesseBrown1980/GitRAM/actions/runs/42936574595 (all three mathematical comparisons PASS; deliberately held by broken archived gzip and an AWK numeric-check error).
- **Final corrected resource-audit cloud run:** https://github.com/JesseBrown1980/GitRAM/actions/runs/42937132424 (all three mathematical comparisons PASS; all 15 timed subprocess RSS gates PASS; every job deliberately red because corrupted original committed gzip remains a REQUIRED release issue).
- Engine-side GPT-6 comparison branch: `gpt6-floor-stageb-full-compare-20261110`, work prior to this receipt through commit `a561bf8ac1862ee5e7c874c61121a7887fe39cfd`.
- Separate Stage B comparison contract frozen before Stage A results at `f8e82f3085a77fd4dc83760ce8a283de9f056245`. The contract was not changed.
- Stage A was authored by a separate Claude seat with a mathematical-only input specification. Its isolation was procedural (same host, model-context hints, and an exposure statement), not a technical sandbox. The separate authorship claim is qualified accordingly. The engine transport and comparison adapter read engine source and are not a third independent mathematical implementation.

## Complete exact row comparison

Each cutoff used a new GitHub-hosted runner. Stage A source rebuilt with Rust 1.81, strict Clippy and `cargo build --release --locked`, then reran all rows and matched the Stage A result-file SHA. Independently reconstructed the pinned original engine rowhash using instrument-only diagnostic export (the diagnostic binary is not the frozen v1 binary) and matched all class rows. Engine class representations were transported to the actual Stage A class lookup, testing the original Stage A Rust and Python label operations for three hand-built configurations (one transposition M=4, one 3-cycle M=6, noncommuting pair M=7), followed by exhaustive bijection and exact comparison.

| m0 | Core classes matched | Tail entries matched | Back moves matched | Exact bijection | All rows |
|---|---:|---:|---:|---|---|
| 11 | 34,920 | 1,738,430 | 7,947,874 | PASS | PASS |
| 12 | 65,025 | 2,977,316 | 14,202,339 | PASS | PASS |
| 13 | 89,301 | 4,181,826 | 22,108,268 | PASS | PASS |
| **TOTAL** | **189,246** | **8,897,572** | **44,258,481** | PASS | PASS |

The exhaustive checker compared `M`, flat moves, class size, source-derived engine class weight, every aggregated core-to-core multiplicity and the complete sorted multiset of `(tail M_k, back core-class multiset)` for **every core class**, after a one-to-one/onto mapping. It also verified the 72-move count for each source row. Each Stage A label-free fingerprint matched the corresponding engine-independent calculation but was treated only as a non-decisive supporting test.

| m0 | Stage A SHA256 of rerun uncompressed rows | Stage A label-free fingerprint |
|---|---|---|
| 11 | `38bc112e3456512334a582de791823c9d8816603c65f2c0f7909966d385c7958` | `899019669d442f36292861c988fe2ace73398353ee2f01fac96f9f9019580098` |
| 12 | `e0c67604816e6cbeb03f6d84a02a02a8b0950beff690c71f4b8ecdf29c6349ca` | `c4551d8eece47a984a26fd9f28175e7df1642f09f84ab880e862d62a9951f87f` |
| 13 | `c9571982b4ac48daa04aa85d0a56756deecabaf58069fb53222fd257a5b68fe7` | `61806ac7b527f4908396dfc28515ae362748e9a8a4f6a71c3d535fe93be52796` |

Engine v1 rowhashes matched `8369f4a9…2884`, `f835b2e1…dee7`, `902be763…654c`; the m0=13 hash is a post-freeze measurement, not a v1 frozen pin.

## External kernel peak RSS gates, repaired cloud run

All entries are in KiB and were taken from `/usr/bin/time -v` on actual GitHub-hosted processes; requirement is strictly below **2,097,152 KiB**.

| Process | m0=11 | m0=12 | m0=13 |
|---|---:|---:|---:|
| Final Stage A source replay | 109,408 | 174,412 | 237,004 |
| 13-letter engine configuration decoder | 19,088 | 19,084 | 18,976 |
| Real Stage A full class lookup | 3,708 | 5,148 | 5,068 |
| Diagnostic engine row reconstruction | 185,340 | 330,324 | 468,980 |
| Exhaustive Python mapped-row comparator | 670,104 | 1,174,508 | **1,560,424** |

**15/15 measured processes under the 2 GiB limit.** The earlier audit step wrongly compared an AWK-extracted RSS string lexicographically and was repaired by `v=$2+0` in `a561bf8...`. The corrected cloud run printed all 5 `RESOURCE|PASS` lines for every cutoff.

## Blocking historical artifact failure

**Every original committed Stage A `results/main/rows-mNN.hbp.gz` fails `gzip -t`, with CRC/length errors.** Git attributes in the Stage A root are `* text eol=lf` and likely normalized binary gzip bytes. The published binary archive is **not usable**, even though all three full rows can be regenerated byte-for-byte from the immutable source and pinned original `results/main/summary-mNN.hbp` SHA256. An archive integrity failure must not be hidden by a successful computational replay.

Transport erratum issue: https://github.com/JesseBrown1980/GitRAM/issues/3

Remedy: preserve the original two Stage A commits and add a new, explicitly labeled erratum commit in the clean-room branch with `*.gz -text` (or binary equivalent), freshly compressed rows made from exact verified source replay, independent `gzip -t`, decompressed SHA256 equal to original `ROWS` pins, and GitHub readback/replay. Do not edit the underlying math source or predictions to repair transport.

**Release status remains HOLD** until this archive fix itself passes an independent published-artifact gate.

## Scope and evidence limits

- The local floor certificate version 2 corrected launcher was not available on GitHub; no launcher acceptance decision was made.
- All Stage B jobs were r-independent: **r=9 computations: zero**. The finite Hamiltonian matrix comparison is NOT proof of the 4D continuum Yang–Mills Clay theorem.
- Six non-equivalent Stage A planted faults triggered checks. Two literal but globally equivalent transformations did not and are disclosed (opposite-group anti-isomorphism and reversed plaquette traversal). No claim that those two were rejected.
- Orbit-level detailed balance and exhaustive representative independence were completed through m0=11; m0=12 and 13 have all-class balance but sampled representative independence. This remains a provenance/coverage qualification, not a discrepancy in the full mapped-class row comparison.
- No reported independent implementation for the second-environment version 2 gate was inspected or executed here.

## Final verdict, measured

`STAGE_B_MATHEMATICAL_COMPARISON=PASS`

`STAGE_B_INDEPENDENT_AUTHORSHIP=PASS_WITH_DISCLOSED_PROCEDURAL_LIMITS`

`STAGE_B_PROCESS_MEMORY_GATE=PASS_15_OF_15`

`STAGE_A_PUBLISHED_ARCHIVE_INTEGRITY=FAIL`

`OVERALL_PUBLIC_ACCEPTANCE=HOLD`

`LOCAL_FLOOR_V2_LAUNCHER=UNREVIEWED_IN_THIS_RUN`

`R9_EXECUTION=HOLD`

`R9_RUNS=0`
