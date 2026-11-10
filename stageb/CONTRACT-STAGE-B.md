# STAGE B COMPARISON CONTRACT — FROZEN BEFORE CLEAN-ROOM RESULTS

Date: 2026-11-10
Owner: GPT-6, engine-side adapter only
Status: CONTRACT FROZEN, STAGE B HOLD
Engine reference: JesseBrown1980/Finite-universe-Yang-mills-existence-and-mass-gap-using-HEB-C at e74d8a46e86034096fd9c9231c87f594f17a93bf
Clean-room Stage A specification: GitRAM commit 93830ad83b8d6bd2047f92f93c9bbde3decd9483, SHA-256 4366313289a3719c538dfb233fb9451854bf3afe5aa0f40d3394269655170389.

## Separation

The implementation of the engine-side decoder and comparison adapter may read the engine's Rust and Python source. The Stage A author must NOT read this branch, the existing engine/verifiers, any engine labels, this contract, known class counts, predicted/observed hashes, or intermediate result values before the two Stage A commits exist. The adapter is not a separate verification of the Hamiltonian; it translates engine data to mathematically specified configurations for an independent program.

No run at r=9 or |r-9|<=1 is authorized; this entire lane is r-independent.

## Defined interface, before seeing Stage A

1. Engine keys are an unsigned 64-bit little-endian file, one key per class, sorted strictly increasing. Each key is the base-6 numeral of thirteen free oriented-link S3 elements under the engine's fixed spanning tree. Read-only adapter decodes to one length-24 list of **explicit permutations of three letters**, not class labels supplied to Stage A.
2. The engine-side canonical *transport* format is line-oriented ASCII:
   `E|key=<decimal>|links=<comma-joined 24 three-digit permutations>`
   where each element `abc` denotes the permutation mapping 0->a, 1->b, 2->c. Link order: h(x,y) for x=0..2,y=0..3, then v(x,y) likewise. Stage A's independent lookup may use another representation; any conversion is limited to permutation and link-order translation.
3. Require an exact class map for every engine key at each m0=11,12,13. The Stage A lookup returns its independently chosen class label, M and core status. The map must be injective AND surjective over all Stage A classes, with a unique engine key for each Stage A label; M must match. Missing label, duplicate label, or noncore lookup is FAIL.
4. Under that established map compare for EVERY row: M, class size, class weight, flat move count; all aggregated core target multiplicities; and the complete sorted multiset of tail entries `(M_k, sorted multiset of mapped core classes reached by 72 second moves)`. Any absent, extra, or unequal entry is FAIL. Histogram-only comparisons are insufficient. Every source row must satisfy `flat + sum(core multiplicities) + tail_count = 72` and class-detailed-balance weighting.
5. The Stage A label-free fingerprint, three colour rounds, is a preliminary cross-check, **not evidence of graph isomorphism**. Both sides use colour_0(A)=decimal `M,size,weight,flat`. At each round sort the complete `colour:count` entries lexicographically and join with comma; sort full tail-entry `M_k:col.col...` strings lexicographically and join with comma; hash UTF-8 of `colour|core|tail` with SHA-256. The fingerprint is SHA-256 of sorted colour_3 hashes joined by newline, with no trailing newline. Matching fingerprint without exhaustive mapped-row equality is UNRESOLVED, not PASS.
6. If the existing engine row serialization lacks class size or weight, derive these by orbit-stabilizer and the 48 mapped orbit representatives; store an independently checked per-key metadata file. Do not change the published Stage A fingerprint. If a reduced fingerprint is necessary, freeze a separate reduced-schema contract and its hash before ANY reduced value is inspected. The original full fingerprint remains required if feasible.
7. The engine-side row serializer has an extra nine-bin tail delta-M histogram; it can be retained as additional evidence but is NOT a substitute for the Stage A tail-back-multiset comparison.

## Input freeze and adversarial controls

- Verify full immutable engine commit, full Stage A commit, every content SHA-256 and Python source SHA-256 before execution.
- Stage A counts/predictions must be committed BEFORE enumeration; enumeration/results and exposure/command receipts in the SECOND commit. No previous data from Stage A may be examined before those two commits.
- Require six independent sabotage controls: change S3 multiplication order, reversed-link inversion, one plaquette orientation, one class size, one core/tail allocation and one tail-back multiset; each must be caught. The engine-side adapter must additionally reject truncated keys, noncanonical link letters, duplicate keys, nonincreasing key order, and every Stage A class collision.
- Require exact integer arithmetic, measured external process peak strictly below 2,097,152 KiB, isolated checkouts and reproducible command logs. A timed out/incomplete job is HOLD, never PASS.
- The published version 1 row counts and hashes and the measured m0=13 hash are used only AFTER the independent Stage A prediction and result commits, for comparison. Do not insert them into the clean-room implementation.
- Only a complete mapping and all-row match can be reported as algorithmically independent software with disclosed procedural-isolation limitations. A matching engine hash alone is a reproduction, not this classification.

## Frozen decision

STAGE_A_SPEC=APPROVED_WITH_DISCLOSURE
ENGINE_ADAPTER=AUTHORISED_TO_BUILD_AND_TEST
STAGE_B_COMPARISON=HOLD_PENDING_TWO_STAGE_A_COMMITS
R9_EXECUTION=HOLD
