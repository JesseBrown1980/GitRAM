# Stage B convention and metadata evidence addendum

Date: 2026-11-10
Status: STAGE B comparison HOLD. The original frozen contract `f8e82f3085a77fd4dc83760ce8a283de9f056245` remains byte-unchanged.
Owner: GPT-6, engine-side only. Not an input to the clean-room Stage A seat.

## Convention known answers before Stage B

Engine transport is a length-24 list in h(x,y) then v(x,y) order (x-major, y-minor); `abc` means the permutation of indices 0->a, 1->b, 2->c. Multiplication is `a after b`, i.e. `(ab)(i)=a(b(i))`. A backward edge has inverse group value.

- Engine key 1: all links identity except v(2,3)=021 (a transposition). Only plaquettes (1,3) and (2,3) are nontrivial, both 021. M=2+2=**4**.
- Engine key 3: all links identity except v(2,3)=120 (a 3-cycle). Plaquette (1,3) holonomy is 120, plaquette (2,3) holonomy is its inverse 201, and M=3+3=**6**.
- Engine key 25194240: h(2,3)=102, v(0,3)=120. The ordered product at plaquette (2,3) is 102 after 120 = 021. Reversing their multiplication incorrectly yields 210. Other nontrivial plaquette holonomies are (2,2)=102 and (0,3)=201; M=2+2+3=**7**.

The engine-side convention harness checks full plaquette words, not just M; M alone cannot distinguish inverse 3-cycles or opposite multiplication orders. Synthetic target encodings cover opposite physical edge orientation, reversed link order, inverse/preimage permutation words and their combination. Once the clean-room seat's own link-order and encoding are frozen, **an end-to-end test through its real translation and lookup is separately mandatory**. Synthetic tests cannot satisfy that final gate.

## Evidence classification of class size and weight

Engine-side `class size` and `class weight` are **DERIVED_FROM_ENGINE / SUPPORTING_METADATA**, because the adapter reads the published Rust engine and may use its orbit-stabilizer convention. For a class representative, the source's `info(key)` computes:

- `norb`: number of distinct gauge orbits under the 48 lattice maps, the class size;
- `nconj`: number of distinct tree-gauge representatives under six simultaneous conjugations, the weight of one orbit in the `6^11` normalization;
- `Wprime=norb*nconj`: the class weight (sum of orbit weights).

These equalities should be checked against independent Stage A values and the exact per-M census, but a match is **not** counted as a separate derivation of the engine's class metadata. The engine-side transport itself does **not** yet compute or export per-key metadata, so the full three-round fingerprint remains on HOLD until that is implemented and checked.

Evidence ranking:
1. **Primary potential independence claim:** every M, flat count, core edge multiplicity and tail back multiset agrees under a proven all-class bijection to independently authored Stage A software.
2. **Corroborating structural evidence:** class-size and weight agreement, global census, and class detailed balance; the engine's side is source-derived.
3. **Non-decisive preliminary evidence:** the label-free three-round fingerprint.
4. **Cross-environment reproducibility only:** the earlier GitRAM `cert_gate.py` hashes and r-independent rows.

A mismatch at *any* requirement is HOLD/FAIL. A matching fingerprint or matching source-derived metadata without the full mapped-row equality is never a successful Stage B verdict.

## Separation and execution

The Stage A author sees none of this until both of its ordered commits are frozen. No coupling-dependent command, including r=9, is part of this preflight. The version 2 launcher is untouched.
