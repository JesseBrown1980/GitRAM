# Engine-side convention gate — measured cloud receipt

Date: 2026-11-10. Stage B comparison remains **HOLD**. Stage A author's inputs unchanged.

- Source branch: `gpt6-floor-stageb-decoder-20261110`
- Known-answer test commit: `2c7370f9a84e2fe61b98223710e4f050df48c273`.
- Evidence addendum: `2afdff7a1281b34840d718acb92d5c1fa6607026`.
- GitHub workflow commit: `dd834f55625ee1f2c42f6d9e522a2d329b1478b4`.
- Harness repair (preserving isolated Python `-I`): `9acc71eea87c512c0916eca754f2673b93a3278e`.
- Accepted cloud run: https://github.com/JesseBrown1980/GitRAM/actions/runs/42927360100
- Job ID: `130217108012`, conclusion **success**.
- Test file SHA-256 (before import-only repair): `7b7d9c7a804681298f74b66d08044fed24238a9adc2f70dabcc1031d527445b4`; after repair, use the owning Git commit and upload artifact manifest rather than this prior hash.

## PASS measurements

| Control | Observed |
|---|---|
| Single transposition on v(2,3) | Two plaquettes have holonomy 021, M=4 |
| Single 3-cycle on v(2,3) | Oriented holonomies 120 and 201, M=6 |
| Noncommuting h(2,3)=102, v(0,3)=120 | Oriented holonomy at (2,3) 021, reverse multiplication would give 210; M=7 |
| Reverse target link slots | Three configuration roundtrips PASS |
| Reverse physical edge directions | Three configuration roundtrips PASS |
| Preimage permutation encoding | Three configuration roundtrips PASS |
| Combined order/orientation/encoding changes | Three configuration roundtrips PASS |
| Deliberately wrong inverse convention | Oriented holonomy mismatch detected |
| Deliberately reversed multiplication | Oriented holonomy mismatch detected |
| Tampered transport permutation | Key-consistency mismatch rejected |
| External measured peak RSS | 17,004 KiB (< 2,097,152 KiB) |

Every check in this table is about the **engine-side decoder and a synthetic translator**. The clean-room seat's actual link-order and encoding have not been received. A later test must translate these known-answer configurations into the real Stage A command and back, checking both exact oriented plaquette holonomies and magnetic M. **No actual Stage A translation or lookup was executed in this run.**

## Failure history and evidence classification

The first cloud run (https://github.com/JesseBrown1980/GitRAM/actions/runs/42927219706) failed before mathematical assertions because `python3 -I` excludes an adjacent-module import. The repair loads that file by explicit pinned path and the final cloud job passed. Do not count the first attempt as a convention failure.

The engine class size `norb` and class weight `norb*nconj` derive from the existing engine's orbit-stabilizer conventions and are **source-derived supporting metadata only**. Even matching metadata and a matching three-round fingerprint do not establish independent equality of the matrices. Stage B still requires exhaustive class bijection and mapped full-row equality; the independent Stage A code and two ordered commits remain pending.

R9_EXECUTIONS=0
STAGE_B=HOLD
ACTUAL_STAGE_A_TRANSLATION=NOT_YET_TESTED
