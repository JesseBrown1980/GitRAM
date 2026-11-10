# Yang–Mills 3×4 S3: independent GitRAM excluded-coupling rehearsal

Date: 2026-11-10. Owner: Jesse Daniel Brown. Independent cloud execution by GPT-6's GitRAM lane.

## Scope and status

This rehearsal checks the preexisting published **v1** freeze source at commit `e74d8a46e86034096fd9c9231c87f594f17a93bf`, **only** at excluded coupling `r = 12`. It does not use the unpushed local v2 commits `cfc57bef` or `3b91d1a0`, modify the historical freeze, execute `r=9`, or establish the continuum mass-gap theorem.

- GitRAM workflow commit: `c416d20e1cda48ccdc87c8927af59c3e78634794`.
- GitHub Actions run: https://github.com/JesseBrown1980/GitRAM/actions/runs/42916130590
- Matrix jobs: `m0=4` (completed), `m0=11` (started; result not filed in this receipt).
- Toolchain: `rustc 1.81.0 (eeb90cda1)`, `cargo 1.81.0 (2dbb1af80)`.
- Rebuilt v1 release binary sha256: `0432b8265b88921e0070592cd996571497511e33584ce5fedc281a28bd358709`.
- **Policy status: HOLD.** The original v1 source exits `101` under strict `cargo +1.81.0 clippy --all-targets -- -D warnings`. The cloud workflow deliberately leaves its overall job red; passing certificate arithmetic is not treated as a passing release.

## Measured excluded-r=12, m0=4

Job: https://github.com/JesseBrown1980/GitRAM/actions/runs/42916130590/job/130178905170

- Core: **73 symmetry classes**.
- `KEYSHASH`: `f57b52ddca435bf9cacd8bbe6a3a3bb3087f578ea1d503d9bf7e58c28dfe3fc0`.
- `ROWHASH`: `e18fddea4fb1a98e2b83ce462c47a892cde1a774ec91e092a2370a13767695a6`.
- Tail `f=1`: **gamma = 599999/10000 = 59.9999**. Independent Python verifier: `VERDICT|PASS`.
- Tail `f=8,7,6,5,4`: **gamma = 884999/10000 = 88.4999**. Independent verifier: `VERDICT|PASS`.
- Tail `f=6,5,4,3`: **gamma = 899999/10000 = 89.9999**. Independent verifier: `VERDICT|PASS`.
- Deliberately altered `rowhash` in certificate: independent verifier rejected; `PLANT|PASS`.
- Highest externally measured process resident peak: **17,376 KiB**, below the explicit **2,097,152 KiB** limit.
- `r=9` executions: **0**.
- Job artifact: `excluded-r12-m4` (run `42916130590`, first archive listing ID `13808614170`).

The three inequalities here concern the finite-model compressed/tail certificate at the excluded control coupling. They are not a numerical answer for `r=9`, nor an accepted mathematical resolution of the Clay continuum SU(N) question.

## Next gates

1. Inspect the full 34,920-class `m0=11` excluded-r control job and its independent verification record.
2. Fix strict Clippy in a *new*, separately identified source revision; don't rewrite the frozen v1 material.
3. Independently freeze and verify exact r=9 commands, two-GiB precedence, per-process timeout, tamper/swap controls, and above/below/undetermined semantics before any `r=9` run.
4. Merge no result into the historical proof record until its own acceptance gate passes.

All statements under **Measured** above were read from the GitHub-hosted job's logs. The job's overall status remains `HOLD` due to strict Clippy failures.