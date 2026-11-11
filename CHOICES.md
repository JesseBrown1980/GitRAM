# CHOICES: where the specification left room

1. **Language.** Rust 1.81 for the enumeration, rows, checks and fingerprint (`src/`, no dependency, no `unsafe`,
   own SHA-256). Python 3 standard library for the independent count (`predict.py`), a second reading of the rows
   files (`verify.py`) and a second labelling function (`label.py`).
2. **Element, site, link and move encoding; canonical label; serialization.** See `FORMAT.md`.
3. **Symmetry action on a configuration.** The image configuration carries U on the image link when the direction
   is kept and U^-1 on the image link when the direction is reversed.
4. **Flat orbits are not core.** "Core = the nonflat classes with M <= m0": the 8 flat orbits (M = 0) are counted
   in the enumeration check but have no row. A move from a tail orbit into a flat orbit is not part of a back
   multiset ("land in the core").
5. **Row weight field.** colour_0 uses the class weight W' = class size times orbit weight.
6. **Fingerprint sorting.** Byte-string order on the complete items (`colour:count`, back colours, `M_k:...`
   entries). Counts and M_k in decimal. Empty lists are empty strings. The final join has no trailing newline.
7. **Tail entries are stored once per distinct content** with a multiplicity (`T` rows). The per-move list of
   the specification is the expansion. Totals count per move: "number of tail entries" = number of (row, move)
   pairs landing in the tail; "total size of all back multisets" = sum over those pairs of the back multiset size.
8. **"Core-to-core nonzero entries"** counts ordered pairs (A, B) with c_(A->B) > 0, the diagonal A = B included.
9. **Independent count.** Character expansion on the torus plus Burnside (formulas in `predict.py`), per M:
   orbits, weight, and the split of orbits by weight 1, 2, 3, 6. It is validated against a brute-force orbit
   count on six small tori (`PREDICT-BRUTE-VALIDATION.hbp`). The enumeration must equal all of it for every
   M from 0 to m0 (check C1), and the class sizes must sum to it (C2).
10. **Enumeration design.** Depth-first search over the 13 non-tree links in tree gauge, in the key order, with
    the partial magnetic number of the plaquettes already closed as the bound; a leaf is kept when its 13 digits
    are the minimum over the six global conjugations (one leaf per gauge orbit); a class representative is an
    orbit whose key is the minimum over the 48 symmetries.
11. **Small cutoffs.** m0 = 3, 4, 6, 7, 8, 9, 10 and also 11: every class, all 48 symmetry images each under
    a pseudo-random gauge transformation plus two pure gauge images (check C4, mode `all`, 50 variants per class),
    and orbit-level detailed balance on every pair of core orbits (C5). C5 is not run at 12 and 13: the pair
    list would not fit under the memory limit with this design.
12. **Sample at m0 = 11, 12, 13 for representative independence** (the production runs in `results/main`).
    Classes with `id % 101 == 50`; three variants each: another orbit of the class (when the class has more than
    one orbit), another configuration of the same orbit (pseudo-random gauge transformation), and both together.
    Generator: xorshift64 seeded from the id. Each variant's whole row (M, size, W, flat, core part, per-move
    tail part) must equal the stored row, and its label must be the same class.
13. **Controls are run at m0 = 7** (682 classes), where every check runs on every class. The three row-level
    controls are run again at m0 = 11 without the representative check.
14. **Control "order of multiplication in S3".** Swapping the order everywhere is the passage to the opposite
    group, which is isomorphic to S3 by inversion and maps the whole construction to itself (U -> U^-1 on every
    link). No check can fail and none does; it is run as mutation 7. The class keys change (the canonical
    representative is read from the inverted configuration), so the rows file differs, and the label-free
    fingerprint and all totals are equal to the unmutated ones. The detectable control (mutation 1) swaps the
    order at one place of use, the propagation `g_b = g_a * U` of the gauge fixing.
15. **Control "orientation of one plaquette's product".** Traversing a plaquette the other way round gives the
    inverse holonomy, which lies in the same conjugacy class, so m_p is unchanged; it is run as mutation 8 and
    the rows file is byte-identical. The detectable controls take the third link forward instead of backward:
    in plaquette 0 = (0,0) (mutation 3) and in plaquette 9 = (0,3) (mutation 9). Mutation 3 sits on a plaquette
    whose other three links are tree links, so in tree gauge it is seen only by the representative-independence
    check C4; mutation 9 sits on a plaquette with four non-tree links.
16. **Row-level controls** act on the finished table: class size of the middle class + 1 (mutation 4); one move
    of one row taken out of a core entry and added to a tail entry (mutation 5); one class id inside one back
    multiset replaced by its successor (mutation 6).
17. **Extra check C7** (not asked for): two-step balance through the tail,
    W'_A T_AB(m) = W'_B T_BA(m), T_AB(m) = number of (move A -> tail orbit with M_k = m, move back into B) pairs.
    It follows from orbit-level detailed balance and is the check that sees a wrong back multiset without
    recomputing rows.
18. **Peak memory** is the kernel high-water mark: `VmHWM` read by the program at exit and
    "Maximum resident set size" of `/usr/bin/time -v`; both recorded. Every run is also started under
    `ulimit -v 2000000` (KiB of address space), so a process cannot pass the limit without failing.
19. **Rows files in git.** The three rows files are committed gzip-compressed (`gzip -n`) under `results/main/`;
    the SHA-256 values are those of the uncompressed bytes.
20. **Timings** are integer milliseconds from a monotonic clock inside the program and the wall clock of
    `/usr/bin/time -v`.
