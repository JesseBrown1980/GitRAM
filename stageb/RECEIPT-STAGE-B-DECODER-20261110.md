# Stage B engine-side decoder — cloud preflight receipt (2026-11-10)

Status: **ENGINE TRANSPORT VERIFIED; STAGE B HOLDS**. Not a clean-room result or an independent reconstruction of Hamiltonian rows. No values from the Stage A seat were read.

## Provenance

- Contract frozen first: `f8e82f3085a77fd4dc83760ce8a283de9f056245`.
- Decoder code added: `fbc4ed9744ba6f43e2ffbc0cbd9b1349b356367d`.
- Cloud test workflow commit: `50ff0e9b414fd0d889822ffa2641a2ef7513eb19`.
- Exact engine input commit: `e74d8a46e86034096fd9c9231c87f594f17a93bf`.
- Decoder sha256: `2e848691fa773e26f7dab33c1a1c0d07a8037171169248d4ff6d98d5c61b2611`.
- Owning GitHub Actions run: https://github.com/JesseBrown1980/GitRAM/actions/runs/42925520916
- All four jobs: `success` at the owning run: one decoder selftest and three full-input transports.

## Measured transport (all no coupling)

| Core m0 | Input keys SHA-256 | Engine class records | Transport SHA-256 | Peak RSS KiB | Verdict |
|---|---|---:|---|---:|---|
| 11 | `e25b8b392161f6f02d933f6a1e78643b3ff95ae3fb69b72b02a36c2c8a5f3a7e` | 34,920 | `d004e17f61d700870a4a6587599e3438eb1d54951074f060c1828d59f00cd10f` | 18,892 | PASS |
| 12 | `942c2eed527aa2c327ce2aa5e811c0c64b0c524a46671f4ccb99ee1086ff5479` | 65,025 | `a6113700cd314ebcea294ca8095de92871b8f1b6caf38f576186e79162bcbac9` | 19,088 | PASS |
| 13 | `b67d54ab32d0c33ac8b1cbd783fa6579c0f7423a342c169a58e4b7c8e86bbc94` | 89,301 | `dc85d9b12deb8922e848568b79f2c72bf935d6f80ad8d9b7d6c23033b39adddd` | 19,020 | PASS |

Every record is the published engine class representative decoded into 24 explicit 3-letter permutations. All keys have strictly increasing binary encoding, valid 13 base-6 letters, identity tree links, and magnetic `0 < M <= m0`. Output is transport only; independent class lookup, class bijection, weighted metadata and full mapped-row comparison have **not** been performed.

## Controls

The decoder `--selftest` passed. On EACH of m0=11,12,13 it rejected four planted inputs: truncated 64-bit key, duplicate key, out-of-range key, and wrong SHA-256 pin. Twelve negative-control executions rejected and none accepted. Strictly less than 2 GiB peak process RSS in all reported measured transports.

## Boundaries

- The engine decoder and contract are on a GPT-6-only branch; neither source nor transport output is an input to the fresh Stage A implementation.
- The code uses group and key encoding observed from the published engine. **It is not a second independent algorithm.**
- Stage B comparison remains HOLD until both Stage A commits and their exposure/command logs are reviewed. Stage B will map every decoded engine representative through the clean-room seat's `class lookup` and compare the full rows, not just a three-round fingerprint.
- The local frozen version 2 and its launcher are untouched.
- `r=9` and all near-`r=9` numerical executions: zero.
