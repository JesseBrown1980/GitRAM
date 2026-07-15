# First deployment receipt — PR #42, run 29415341620 (MEASURED 2026-07-15)

All facts below were read live from the owning gate (GitHub API / `gh`) on 2026-07-15
by seat ACER-CLAUDE-FABLE5. Tags: MEASURED = read from GitHub; UNVERIFIED = not proven.

## Where

- Repo: `JesseBrown1980/HYPER-BECHS--the-third-set`
- PR: **#42** — "GitRAM: train Pais Omega floor one across 27 reversible cubes" (draft, open, mergeable)
- Branch: `agent/liris-pais-gitram-floor1-20260715` (LIRIS seat) — one commit `94718f8f78ea`
- Payload: `.github/workflows/pais-omega-floor1-gitram.yml` (the GitRAM lane),
  `pais_omega_floor1.rs` (545-line dependency-free Rust trainer), 11 SHA-pinned Pais
  patent PDFs, HBP corpus manifest + floor contract + sidecars, `GITRAM.md`.

## Run 29415341620 — "Pais Omega floor one GitRAM (27 × 800)"

| Stage | Result | Detail |
| --- | --- | --- |
| 27 × `Cube N / 800 Cartesian cells` | **27/27 success** (MEASURED) | each ~1.5–3.5 min; matrix `max-parallel: 20`, `fail-fast: false`, per-cube timeout 360 min |
| Artifacts | **27/27 uploaded** (MEASURED) | `pais-cube-1` … `pais-cube-27`, ~0.98–1.42 MB each, retention 30 days |
| `receipt integrity` (companion run 29415341554) | success (MEASURED) | |
| `Verify 27 checkpoints and seal floor-one Omega` | **cancelled** (MEASURED) | started 12:31:58Z, killed 12:54:50Z (~23 min) |

### Fan-in cancellation analysis

- Log tail (MEASURED): sequential verify emitted `CUBE_OK` for cubes **01–14** (each
  ~1.2–1.9 min, e.g. `cube=01 cells=800 accepted=365 held=435 gain=57067642`,
  `cube=02 … accepted=772 gain=67907293`), then at cube 15:
  `##[error]The runner has received a shutdown signal.` → exit code **143** (SIGTERM).
- **Not a timeout**: the job's `timeout-minutes: 60` was not reached (~23 min elapsed).
- **Not a concurrency cancel**: the workflow sets `cancel-in-progress: false`.
- Actual cause (manual cancel vs. infrastructure): **UNVERIFIED**.
- Projected full fan-in duration at the observed pace: ~27 × ~1.5 min ≈ **40 min**,
  within the 60-min limit — a plain re-run of the fan-in job should complete.

## Consequence

**The floor-one Omega is NOT yet sealed on the cloud.** All 27 trained checkpoints are
preserved as artifacts (30-day retention from 2026-07-15). The seal is one fan-in
re-run away ("Re-run failed jobs" on run 29415341620, or a fresh dispatch); the fan-in
performs no duplicate training. The seal gate is
`PAIS_FLOOR_PASS|cubes=27|cells=21600` plus `sha256sum -c` over the fan-in tree.

## Companion contract

- `Metatagging-data-for-a-Quantum-universe` **PR #5** — "Add PID-traceable Floor-1
  Omegaverse preparation" (same day, 17:52Z, LIRIS CDC branch). CI green 3/3
  (Python 3.11/3.12/3.13, run 29438624158) + "Floor-1 Omegaverse verification" success.
- Cross-link comment on PR #42 (17:58Z) binds the evidence boundary: **PR #42 owns any
  actual Floor-1 training results**; PR #5 reports improvement claims = 0, glyph
  semantic evaluations = 0, external GNN training = 0, process launches = 0, and
  fabric writes = 0.
