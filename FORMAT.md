# Stage A: encoding, canonical class label and row serialization

Everything here is integer and text. Lines end in LF. HBP rows are `TAG|k=v|...|json=0`; the bar is only a separator.

## 1. Encoding of a configuration

* Group elements are digits 0..5, permutations of {0,1,2} written as the images of 0,1,2:
  `0 = e = [0,1,2]`, `1 = (0 1) = [1,0,2]`, `2 = (0 2) = [2,1,0]`, `3 = (1 2) = [0,2,1]`, `4 = [1,2,0]`, `5 = [2,0,1]`.
  Product: `(a*b)(i) = a(b(i))`. Digits 1,2,3 are the transpositions, 4,5 the 3-cycles.
* Site index `s = x + 3*y`, x in 0..2, y in 0..3.
* Link index `l = 2*s + d`; `d = 0` is the link (x,y)->(x+1,y), `d = 1` is the link (x,y)->(x,y+1).
* A configuration is 24 digits, link 0 first.
* Plaquette at (x,y): holonomy `U[x-link(x,y)] * U[y-link(x+1,y)] * U[x-link(x,y+1)]^-1 * U[y-link(x,y)]^-1`;
  m = 0, 2, 3 for e, transposition, 3-cycle; M is the sum over the 12 plaquettes.
* Move number `mv = 3*l + (t-1)`, t in 1..3: `U[l] -> t * U[l]`.

## 2. Canonical label

1. **Tree gauge.** Tree links: x-links (0,0)->(1,0) and (1,0)->(2,0), and the y-links (x,y)->(x,y+1) for y = 0,1,2
   and all x. Set g at site (0,0) to e and `g_b = g_a * U_(a->b)` along the tree; the gauge transform by g makes
   every tree link e. The 13 other links then hold `v = g_a * U_(a->b) * g_b^-1`, read in this order:
   x-link at (2,0); x-links at (0,1),(1,1),(2,1); (0,2),(1,2),(2,2); (0,3),(1,3),(2,3); y-links at (0,3),(1,3),(2,3).
2. **Orbit key.** For each of the six c in S3 form the base-6 number with digits `c * v_i * c^-1`, first link most
   significant. The orbit key is the smallest of the six. The stabiliser order is the number of c that leave the
   digits unchanged; the orbit weight is `w = 6 / stabiliser` (1, 2, 3 or 6).
3. **Class key.** Apply each of the 48 lattice symmetries (x,y) -> (sx*x + tx, sy*y + ty), sx, sy = +1 or -1:
   a link whose direction is kept carries its element to the image link; a link whose direction is reversed carries
   the inverse element to the image link (the image link starts at the image of the far end). The class key is the
   smallest orbit key over the 48 images. Class size = number of distinct orbit keys among the 48. `W = size * w`.
4. **Class id.** The nonflat classes are ordered by (M, class key) ascending and numbered from 0. The order does
   not depend on the cutoff, so the ids at m0 = 11 are a prefix of the ids at 12 and 13.

The class key is the label: it is defined for every configuration without any table. The id needs the rows file.

Commands (item 6 of the specification):

    target/release/cleanrows label <24 digits> [rows-mNN.hbp]
    python3 label.py <24 digits>

Both print `LABEL|class_key=..|M=..|size=..|w=..|W=..|flat=..|core_m11=..|core_m12=..|core_m13=..|...|json=0`;
`core_mNN=1` when 0 < M <= NN. The Rust command adds `id=` when a rows file is given.

## 3. Rows file `rows-mNN.hbp`

Three kinds of rows, in this order.

    HEAD|stage=A|m0=<m0>|classes=<n>|tailsets=<nt>|json=0
    T|t=<tid>|M=<M_k>|n=<back size>|back=<id>:<count>,<id>:<count>,...|json=0
    R|id=<id>|key=<class key>|M=<M>|size=<size>|w=<w>|W=<W>|flat=<flat>|ncore=<c>|ntail=<t>|core=<id>:<count>,...|tail=<tid>:<count>,...|json=0

* A **tail set** is the pair (M_k, multiset of core class ids reached by the moves of the tail orbit k that land in
  the core). Identical tail sets are stored once. Tail sets are ordered by the integer sequence
  `[M_k, id, id, ...]` (ids ascending with multiplicity; lexicographic comparison; a proper prefix sorts first) and
  numbered from 0. `back=` lists id:multiplicity with ids ascending; `n` is the multiset size.
* An **R row** is the row of one core class. `flat` = moves landing in a flat orbit; `core=` lists
  core class id : number of moves, ids ascending; `tail=` lists tail set id : number of the 72 moves whose tail
  orbit has that tail set, tids ascending. `ncore` and `ntail` are the sums of the counts; `flat + ncore + ntail = 72`.
  The specification's "one entry per move" list is obtained by repeating each tail set `count` times.
* An empty list is written `-`.
* R rows are in id order. The file SHA-256 is taken over the exact bytes.

## 4. Summary file `summary-mNN.hbp`

`PERM` rows: per M classes, orbits, weight, the prediction, cumulative totals. `HIST`: class-size histogram.
`TOTALS`: section-3 totals. `ROWS`: file hash. `FINGERPRINT`: label-free fingerprint and distinct colour count.
`CHECK`: each named check with PASS or FAIL. `RUN`: timings in milliseconds and the kernel high-water mark.

## 5. Fingerprint

As in the specification. colour_0 = `M,size,W,flat`. In each round the items `colour(B):count` are sorted as
byte strings and joined by `,`; each tail entry (one per move) is `M_k:` followed by the back colours sorted as
byte strings, with multiplicity, joined by `.`; the tail entries are sorted as byte strings and joined by `,`.
Empty lists give the empty string. Three rounds; the fingerprint is the SHA-256 of the sorted colour_3 values
joined by LF with no trailing LF.
