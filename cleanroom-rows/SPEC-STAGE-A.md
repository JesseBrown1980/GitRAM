# Stage A specification: clean-room row reconstruction, periodic 3x4 S3 lattice

This file is the whole input of the Stage A author. It holds the mathematical definition and the output contract.
It holds no expected counts, no hashes and no implementation detail of any existing program.

## 1. Definition

* **Lattice.** Sites (x, y), x in Z/3, y in Z/4: 12 sites. Each site carries two oriented links, (x,y)->(x+1,y) and
  (x,y)->(x,y+1): 24 links. Each site carries one plaquette (the unit square with that site as lower-left corner): 12.
* **Group.** S3, the permutations of three letters: identity e, three transpositions, two 3-cycles.
* **Configurations.** U in S3^24, one element per oriented link; traversing a link backwards uses the inverse.
* **Gauge transformations.** g in S3^12 acts by U_(a->b) -> g_a U_(a->b) g_b^-1. A **gauge orbit** is an orbit of
  this action. The **weight** w of an orbit is (number of configurations in it) / 6^11; the weights of all orbits
  sum to 6^13.
* **Magnetic number.** The holonomy of a plaquette is the ordered product of its four link variables around the
  square. Its conjugacy class is gauge invariant. m_p = 0 for e, 2 for a transposition, 3 for a 3-cycle
  (m_p = 2 - chi_std). M = sum of m_p over the 12 plaquettes. An orbit is **flat** when M = 0.
* **Moves.** 72 moves (link e, transposition tau): U_e -> tau U_e, every other link unchanged. For gauge orbits
  i, j: c_ij = number of the 72 moves that take a configuration of orbit i into orbit j (the same for every
  configuration of i; self-moves count).
* **Lattice symmetries.** 48 maps: the 12 translations composed with the reflections x -> -x and y -> -y. They
  permute links, inverting the variable on every link whose orientation is reversed. They map gauge orbits to
  gauge orbits. A **class** is an orbit of gauge orbits under these 48 maps. Class size = number of gauge orbits
  in it; class weight W' = sum of their weights.
* **Core and tail at cutoff m0.** Core = the nonflat classes with M <= m0. Tail = everything with M > m0.
* **Row of a core class A** (taken at any one configuration of any one orbit of A; show that the result does not
  depend on that choice):
  * M_A, class size, class weight;
  * flat count: how many of the 72 moves land in a flat orbit;
  * core part: for each core class B, the number of the 72 moves that land in B;
  * tail part: for each of the 72 moves that lands in a tail orbit k (one entry per move): M_k, and the multiset
    of core classes reached by those of the 72 moves of k that land in the core.

## 2. Task

Build, from section 1 alone, a program that enumerates the core for m0 = 11, 12 and 13 and reconstructs every
row. Exact integers only: no float type, no float literal, no division producing a float, anywhere. Python 3
(standard library only) or Rust 1.81 with clippy -D warnings and no `unsafe`. No coupling value is needed or
used. Measured peak memory of every process stays below 2,097,152 KiB (take the kernel high-water mark, e.g.
`/usr/bin/time -v` or `resource.getrusage`, and record it).

Completeness must be shown by your own independent argument: derive a closed-form or otherwise independent count
of gauge orbits and total weight per value of M (for example by character theory or a counting method that does
not share code with the enumeration) and require the enumeration to equal it for every M up to the cutoff.

## 3. Output (your own format, documented by you)

For each m0 in {11, 12, 13}:
1. per M: number of classes, number of gauge orbits, total weight; cumulative totals;
2. histogram of class sizes;
3. totals: core classes, core-to-core nonzero entries, sum of flat counts, number of tail entries, total size of
   all back multisets;
4. the full rows in a canonical text serialization of your own design with its SHA-256, and a short document that
   specifies that serialization and your canonical class labelling completely;
5. a **label-free fingerprint**, so that a party with different class labels can compare:
   colour_0(A) = the string `M,size,weight,flat` in decimal;
   colour_(t+1)(A) = sha256 hex of the UTF-8 string
   `colour_t(A)|` + sorted list of `colour_t(B):count` over core B joined by `,` + `|` + sorted list over tail
   entries of `M_k:` + (sorted `colour_t` of the back classes, with multiplicity, joined by `.`) joined by `,`;
   three rounds; the fingerprint is the sha256 hex of the sorted list of colour_3 values joined by newline.
   Also give the number of distinct colour_3 values;
6. a function or command that takes any configuration (24 elements in a link order and element encoding you
   document) and returns your class label, its M and whether it is in the core.

## 4. Controls (each must be detected by a check you name)

Mutate, one at a time: the order of multiplication in S3; inversion on reversed links; the orientation of one
plaquette's product; one class size; the allocation of one move between core and tail; one back multiset.
Also check: w_i c_ij = w_j c_ji on every core pair at orbit level for the smallest cutoffs and at class level
(W'_A c_(A->B) = W'_B c_(B->A)) on every core pair; row independence from the representative (another orbit of the
class, another configuration of the orbit) on a sample you state, and on every class for a small cutoff.

## 5. Clean-room rules

* Do not open, list, search or fetch: any path containing `floor`, `ym`, `Yang`, `finite-universe`, `GitRAM`,
  `cert_gate`, `indep_verify`, `floorcert`; any GitHub repository, issue, pull request or Actions log; any memory
  or recall entry about this lattice. Use no web search. Work only inside your own directory.
* Keep `EXPOSURE.md`: what you had seen before starting (including anything in your injected context), every file
  you opened outside your directory, and a statement of what you cannot show.
* Keep `COMMANDS.log`: every command you ran, in order.
* Write the per-M predictions you derive by the independent count into `PREDICT-STAGE-A.hbp` (rows
  `TAG|k=v|...|json=0`) and commit it **before** the full enumeration is run; then run, then commit the results in
  a second commit. Two commits, in that order, author `Jesse Daniel Brown <plasmatoid@gmail.com>`, no trailers,
  no push.
* If something in this specification is ambiguous, choose, record the choice in `CHOICES.md`, and continue.
