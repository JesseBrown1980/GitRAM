# Finite Yang–Mills cloud independence receipt — November 10, 2026

Operator: Jesse Daniel Brown. This report is for a **published version 1, excluded-coupling or r-independent control**, not the local unpushed version 2 freeze or any r=9 measurement.

## Provenance and independence definitions

**Writer:** `work/design/rs/src/main.rs` from the published v1 commit `e74d8a46e86034096fd9c9231c87f594f17a93bf`, rebuilt as Rust 1.81 release under GitRAM.

**Python rows-only checker:** `work/design/py/cert_gate.py` from **that same historical v1 repository**, executed on separate GitHub-hosted Ubuntu runners. This Python implementation is independently written relative to the Rust writer, but is **NOT a newly authored third implementation**. Running the same Python code locally and in cloud verifies environment reproducibility, not code-development independence. The reported local match and first-verifier match are secondhand until their own logs are inspected.

## Measured GitHub results

| Core | Classes | Pinned key hash | Independently rebuilt Python row hash | Memory (kB) |
| --- | ---: | --- | --- | ---: |
| m0=11 | 34920 | e25b8b392161f6f02d933f6a1e78643b3ff95ae3fb69b72b02a36c2c8a5f3a7e | 8369f4a981a888f4e028b155f3306371c2eacb421bed9bbc444b86e7ef862884 | 401096 |
| m0=12 | 65025 | 942c2eed527aa2c327ce2aa5e811c0c64b0c524a46671f4ccb99ee1086ff5479 | f835b2e1ded4b7f3f38d9d7f3f8437ebfa0a6ede53783f224ebbd1e9b4b7dee7 | 734036 |

Both row hashes match v1 prediction pins 028 and 029 exactly. These are **r-independent row reconstructions**; no r=9 calculation has run.

Source log: https://github.com/JesseBrown1980/GitRAM/actions/runs/42917513203

Additional excluded-coupling evidence from that run:
- r=21/2 at m0=4: three written certificates independently checked by the preexisting Python checker, all VERDICT|PASS.
- Two planted mutations, one to rowhash and one to the tail cap, rejected.
- Last process peak 17,176 KiB.
- Overall excluded-control job remains RED by design because the published v1 source fails strict Rust 1.81 Clippy. Its mathematical certificate subprocess and sabotage tests passed.

First Rust run: https://github.com/JesseBrown1980/GitRAM/actions/runs/42916130590
- m0=11, r=12, lower bound 1044452/10000; Python checker VERDICT|PASS.
- Rust and Python m0=11 rowhash agree with full pinned hash 8369f4a981a888f4e028b155f3306371c2eacb421bed9bbc444b86e7ef862884.
- Maximum peak 440,616 KiB, planted rowhash fault rejected.
- Workflow overall red due to known v1 Clippy failure; no r=9 claim.

## Newly launched m0=13 experiment

Run: https://github.com/JesseBrown1980/GitRAM/actions/runs/42920725746

- Separate Rust rowhash writer and Python cert_gate.py rows-only verifier on distinct GitHub runners.
- 89,301 classes, pinned `core_keys_m13.bin` sha256 `b67d54ab32d0c33ac8b1cbd783fa6579c0f7423a342c169a58e4b7c8e86bbc94`.
- Its row hash was **not pinned in v1**. Local verifier reports prefix `902be763`; the GitRAM workflow must report and compare the **complete 64-hex hash** from both implementations. This receipt does not claim the m13 job passed before its final results exist.
- Exact-integer cross-multiplication exclusion-unit-tests have passed. The m13 jobs execute **no r-dependent command**. The same exclusion gate must later be bound to the actual v2 invocation entrypoint; a unit-test alone does not protect an arbitrary new script.
- The old v1 source's strict Clippy HOLD remains. This cannot be used to approve the local v2 freeze.

## On the future v2 push

Only after the revised local v2 is published and verified, prepare a second-environment gate that:
1. Checks out the immutable **full commit SHA** and verifies `FREEZE-PIN.sha256` before using any file.
2. Rebuilds with locked dependencies under Rust 1.81, checks strict Clippy, and compares the binary to **row 016**.
3. Independently checks **each published certificate** by executing the pinned `gate_required.sh` directly from a clean checkout, with the same certificate SHA-256 as the launcher's receipt.
4. Refuses all `r` with `|r-9| <= 1` by exact rational integer arithmetic. Spellings including `9/1`, `18/2`, `90/10` and `19/2` must all be rejected. No r=9 calculation or logs of its computed values are authorized.
5. Never treats the launcher's own PASS line as verifier evidence; verdict fields and certificate hashes must be reproduced in the independent environment.

**Status:** partial finite, excluded-coupling evidence accepted within stated scope; local v2 remains unpushed and on HOLD.
