# Floor Two contract — bigger glyph set, trained from the floor below

Status: **DESIGN_OPEN** (no training has run; no results are claimed).
Authority: floor two unlocked by operator OP-JESSE, 2026-07-15 — *"Continue to floor
two with the bigger language set now."* Floors three and above remain **HELD**.

## The sizing answer: bigger symbols, smaller streams

Each floor-two cube uses a **BIGGER symbol set** than the floor below — and consumes a
**SMALLER, denser stream**. The two move together; that is the whole mechanism of the
ladder:

- **Alphabet: 256 → 1024 glyphs (4×).** Floor one trained on the raw byte alphabet
  (256 symbols, 8 bits). Floor two tokenizes floor-one *trained motifs* into 10-bit
  glyphs (1024 symbols). This is not a new invention — it is the system's own existing
  ladder step (BEHCS-256 → BH-1024), so floor two **binds to the existing BH1024 glyph
  catalog** rather than minting new shapes. Letters vs. shapes vs. symbols is a render
  question only: a glyph is a 10-bit index with a catalog face.
- **Why 4× and not more:** the floor-one seal measured `accepted=13339` trained cells
  across the floor. Spread over 1024 glyphs that is ~13 trained exemplars per glyph —
  healthy support. Over 4096 glyphs (16×) it would be ~3.3 per glyph — too sparse to
  train; the alphabet would outrun the floor below. 4× is the largest step the measured
  floor-one density supports.
- **Why not smaller:** shrinking the alphabet at a higher floor re-expands the streams
  and throws away the structure floor one already extracted — the deflationary
  direction. Every prior stratum of the system grows the representational alphabet
  upward (BEHCS-256 → BH1024; catalog 47D → 60D) while the lower layer stays alive as
  the decode bridge.
- **Old decodes new:** every floor-two glyph must decode exactly through the floor-one
  byte layer. Floor one is not discarded — it is the bridge stratum. Reversibility and
  byte-exact restore of the original 11-PDF corpus must survive through both floors.

## Input: the floor below, not the raw corpus

Floor-two cubes do **not** re-read the 11 Pais PDFs. They train from the floor-one
trained bodies: the 27 sealed cube checkpoints + the floor-one Omega
(`pais-omega-floor1-sealed`, artifact 8355679931 of run 29415341620, omega
`cb8f8e7e0d30fad7079fe1d887f613fcc3154de704a0ab0af7fbdf5a52840d2e`, result sha256
`5811cf81…de63db`). Raw intake per cube shrinks; information per symbol rises.

## Shape and schedule: change exactly one variable

- **Bodies: 27 base + 6 apex + 1 Omega = 34** (the corrected floor law), with the
  6-of-6 apex formation gate.
- **Schedule: unchanged** — 8 reversible representation lanes × 10 predictor functions
  × 10 persistent epochs = 800 cells per cube.
- Only the **alphabet** changes at this floor (256 → 1024). Holding the schedule fixed
  keeps floor-two gains attributable to the bigger language and comparable cell-for-cell
  against floor one's 13,339/21,600 accepted baseline.

## GitRAM lane

Floor two runs as a GitRAM lane per this repo's template: 27 stateless cells (one per
floor-one checkpoint) + apex/Omega fan-in, artifacts as the memory bus, all-or-nothing
seal, `cancel-in-progress: false`, fan-in timeout sized for ~34 × per-body verify time.

## The BEHCS-64 control arm (operator prompt, 2026-07-15)

BEHCS-64 was never trained — not because it is "old," but because floor one's raw
input is already natively the 256-byte alphabet; 64 sits *below* the raw stratum. It
still confirms the ladder: **64 → 256 → 1024 is ×4 per step**, exactly the step this
contract takes. To make the sizing law measured rather than argued, floor two carries a
cheap **64-glyph control arm**: one shadow cube trained on the same floor-one input with
a 4×-SMALLER alphabet (1024 → wrong direction → 64). Prediction under this contract:
its accepted-cell rate and gain collapse relative to both the 1024 arm and the floor-one
baseline. If the 64 arm ties or wins, the bigger-symbols law is falsified and floor two
re-designs before sealing. GitRAM makes this control one extra container.

## Open items (blocking training start, not blocking this contract)

1. **±R/±N/±Q binding is unresolved** (2026-07-15 correction: they are route labels;
   canonical apex roles are ±X/±Y/±Z). The floor-two trainer must not bake the
   unresolved labels in deeper — binding to be confirmed by the fabric/owning seats.
2. **Trainer implementation** — extension of the floor-one dependency-free Rust trainer
   to 10-bit glyph streams; LIRIS-seat parity or an acer-seat branch, coordinated
   through the GitHub mediator (PR #42 lane).
3. **No training claims** until the owning gate (the floor-two GitRAM run's own checks)
   concludes — same claims discipline as floor one.
