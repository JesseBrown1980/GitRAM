# YM4-06C — actual finite 3D S3 spectral calculations

Date: 2026-11-11. Scope: periodic 2×2×2 **three-dimensional spatial** lattice with exact S3 gauge group. The Hamiltonian acts on the complete gauge-invariant state space (2,821,153,019,713 gauge orbits), but no matrix of that size is stored. The result is **not** the continuum SU(2) physical mass gap.

## Two targeted gaps

For ordered physical eigenvalues of H(r)/J_E = 72I − T + rM:

- first excitation: d1 = e2 − e1;
- flat-cluster separation: dflat = e22 − e21, above the 21 flat magnetic gauge sectors.

The run contains a **genuine exact 21×21 fourth-order effective spectral matrix**, rather than a generic positivity proof. The distinction between fourth-order eigenvalue coefficients and full-H spectral enclosures is maintained in every receipt.

### Exact fourth-order spectrum

In the rooted spanning-tree gauge there are 17 free S3 links. All 48 rooted flat configurations are reconstructed from commuting triples, and quotient under residual global conjugation gives the 21-dimensional physical flat space. For each rooted flat vector, traverse **two actual electric moves**, excluding returns into the flat set. Store the 120,888 distinct two-hop rooted states and form a weighted Gram matrix using their exact magnetic costs m in {8, 11, 12, 14, 15, 16}. The magnetic state weights are 1/m and the two boundary denominators are each 1/8. All arithmetic is exact integer with common denominator L = 1,182,720. This is the two-step matrix-jump factorization, using no full Hilbert-space matrix.

The resulting physical 21×21 integer matrix C = L W4 has maximum absolute row sum 14,021,568 and factorized characteristic polynomial

\[
 \det(xI-C) =
 (x-13112208)^4 (x-13138128)^6\,P_3(x)^2 P_5(x),
\]
\[
 P_3(x)=x^3-39492144x^2+519816640269312x-2280439028453407137792,
\]
\[
\begin{split}
 P_5(x)=&x^5-65820240x^4+1732733020247040x^3\\
 &-22804817983658137681920x^2\\
 &+150052887866985564407416750080x\\
 &-394888873798641922702044531385171968.
\end{split}
\]

**Why the factorization is exact, not a numerical guess:** The native checker calculates determinants of xI−C at 22 distinct x-values modulo each of 20 independently trial-divided primes greater than 10^9, and matches the factorized polynomial every time. This establishes coefficient equality modulo each prime (degree 21). Since 20 distinct prime moduli have a product greater than 10^180, while Gershgorin and the proposed factors each bound all coefficient magnitudes by 10^158, the coefficients are identical as integers. Integer sign changes in 3 disjoint intervals for P3 and 5 for P5 isolate all real roots.

The largest two **physical** fourth-order coefficients satisfy

\[
 \frac{13582885}{1182720}<\lambda_1<\frac{13582886}{1182720},
 \qquad
 \frac{13412285}{1182720}<\lambda_2<\frac{13412286}{1182720}.
\]

Thus

\[
 \frac{170599}{1182720}
 < \lambda_1-\lambda_2
 < \frac{170601}{1182720}.
\]

At r=27 and r=81 the corresponding first-gap values at leading fourth order are approximately 7.33×10^-6 and 2.71×10^-7 in J_E units, respectively. These are **exploratory asymptotic calculations**, NOT certified full-H intervals at those couplings. The coefficient was explored before this branch was published and is **not claimed to be a blind prediction**.

### Full-H first excitation at ternary coupling r = 2187

For r>12, let P project onto the full rooted flat space, Q=I−P, and put z=72−s. The exterior Schur resolvent exists because M>=6 on Q and ||T||<=72. The Feshbach self-energy is

\[
 \Sigma(s)=PTQ\,[rM+s-QTQ]^{-1}QTP.
\]

The 48 rooted flat vectors all have the same electric parity. The electric T flips that parity, so odd powers in the Neumann expansion vanish. Since every one-hop flat neighbor has M=8 and the exact single-hop Gram is 72I,

\[
 \Sigma(s)=\frac{72}{8r+s}I+K_4(s)+R_{\ge6}(s).
\]

With q=12/r and alpha=9/r, the even tail is positive semidefinite and is bounded in operator norm by epsilon = alpha q^4/(1−q^2); each low-level shift s is between 0 and smax = alpha/(1−q^2). The fourth-order matrix obeys

\[
 \left(1-\frac{s_{\max}}{2r}\right)\frac{W_4}{r^3}
 \ \preceq\ K_4(s)\ \preceq\ \frac{W_4}{r^3}.
\]

For the ordered first two **physical** roots s1 >= s2 of the self-consistent equation s=Sigma(s), define f(s)=s−72/(8r+s). Its derivative is between 1 and 1+72/(8r)^2, so the exact isolated lambda1 and lambda2 intervals, the small variation above, and the positive tail bound yield an explicit full-H first-gap enclosure.

At **r=2187**, conservative *integer-cross-multiplied* inequalities in the native code prove

\[
 \boxed{\frac{1}{120000000000}
 < \frac{E_2-E_1}{J_E}
 < \frac{1}{50000000000}.}
\]

The finite first gap at this specific large coupling is therefore between approximately 8.33×10^-12 and 2.00×10^-11 J_E. The lower and upper statements apply to the **full gauge-invariant Hamiltonian** on this finite torus, not just the fourth-order truncation. Independent mathematical review remains appropriate.

### Full-H flat-cluster enclosure using the complete M=6 manifold

The native constraint solver fixes a spanning tree and enforces trivial plaquette holonomy outside every possible 2-face winding tube support. There are exactly 12 such supports, each giving 18 rooted configurations: **216 M=6 configurations, 36 physical conjugacy orbits**, every orbit of size 6. All 15,552 candidate one-link moves (216×72) are checked: **none connects two M=6 configurations**. The earlier YM4-06A support-completeness argument says that below magnetic cost 8 there are only M=0 and M=6; there is no M=7.

Let P6 project onto all M=6 configurations and P0 onto the 21 flat configurations. Then (P0+P6) H (P0+P6) is exactly diagonal by these two energy blocks:

\[
 H_{P0}=72I,\qquad H_{P6}=(6r+72)I.
\]

Its exterior Q has M>=8, and E>=0. Schur inertia, ||T||<=72 and min-max then yield, for r>36,

\[
 6r-\frac{5184}{2r-72}
 < \frac{E_{22}-E_{21}}{J_E}
 \le 6r+\frac{9r}{r^2-144}.
\]

These are **two-sided bounds on the full physical flat-cluster gap**, not merely a classical magnetic lock cost.

- At r=81: 428.4 < dflat/J_E <= 486.113605.
- At r=2187: 13120.794979 < dflat/J_E <= 13122.004116.

At r=27 the earlier lower bound 90 J_E still applies; the r>36 two-sided argument does not.

## Implementation and integrity

- Rust 1.81 strict Clippy, no floating point, zero JSON, no JS/Node/V8 in the authored computation or hosted workflow, no extra Rust dependencies.
- S3 is represented by affine transformations on Z3; all 17 free-link addresses are **two constrained trits per link** in powers of 9 (3^2).
- A native integer SHA-256 implementation produces HBP/HBI/SHA/SH/HASH receipts; 21 complete matrix rows and the proof's root brackets are printed in the hosted log.
- The hosted workflow checks the complete reconstructed Gram matrix and 20-modulus exact characteristic polynomial, separate M6 constraint enumeration, strict Clippy and regression tests, and a deliberate false-denominator control.
- The full 300 GB physical Host8 store is not read; GitRAM's previously verified 20-worker storage test is a separate experiment. The finite spectral calculation uses the matrix laws to **avoid** needing that store.
- The separately frozen r=9 one-shot is not executed. No spacing-dependent transfer operator or SU(2) continuum mass is calculated here.

For independent acceptance: replay both native solvers, check the CRT size proof and Feshbach/spectral min-max argument separately, attack the support-completeness assumption from YM4-06A, and keep the numerical high-coupling result distinct from a 4D continuum claim.
