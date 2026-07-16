# Spatial playback — the unbuilt six-direction lane

Status: **UNBUILT / HELD** (operator-gated, like every new lane). Recorded 2026-07-15
from the LIRIS ledger so the cloud record matches what exists.

## The precision that motivates it

Floor one's six signed directions ±R/±N/±Q are exact replay gates, and its own
receipts prove they **balance in pairs**: `same_transform_for_sign=1` — R, N, Q are
involutions, so +R and −R are byte-identical transforms. Six signs, three distinct
transforms. The same structure was built twice on 2026-07-15: in
`Metatagging-data-for-a-Quantum-universe` PR #5 (`CUBIC_6_APEX` apex bodies, omnibit
Omega binding, `APEX-HUMAN-JESSE` as the only seat with a final-apex claim) and as
floor two's six apex bodies.

(Floor two's apex bodies do **not** pair-collapse — they are lattice traversal orders,
not signed transforms: the −axis order flips coordinate groups but keeps within-group
ties ascending, so −X ≠ reverse(+X); the trainer's selftest asserts all six orders
distinct. But they are traversals of a 27-body lattice, not of the bytes themselves.)

## The unbuilt construction

Fold a body's bytes into a literal spatial cube — e.g. 27,000 bytes → 30×30×30 — and
play it back **x-major, y-major, z-major, each forward and backward**. Axis-major
transposes of a 3D array are genuinely different streams in all six directions: no
pair balances, because a forward and a backward pass over a transposed volume really
are different byte orders. Six true read orders per body.

## Where it would slot

Six additional reversible representation lanes (fold → axis-major read is exactly
invertible given the cube dimensions), extending Ring-A from 8 to 14 lanes — a later
floor or a dedicated experiment lane, run as GitRAM cells like everything else.

## Gates before it fires

- Operator unlock (new lane = new scope; floors 3+ HELD).
- Must NOT touch the staged floor two — its seat-convergence leaf predictions
  (`8dd67d8e…`, `613fd954…`) stand unmodified until the CI run tests them.
- Same claims discipline: receipts + owning gate, no record claims.
