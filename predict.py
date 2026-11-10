#!/usr/bin/env python3
# Independent count of gauge orbits and total weight per magnetic number M.
# Character theory only; shares no code with the enumeration. Exact integers only.
#
# Torus with N plaquettes, group G, class function f(g) = t^m(g) on each plaquette:
#   Z_G(t) = |G|^E * sum_r (a_r / d_r)^N,  a_r = (1/|G|) sum_g f(g) chi_r(g)
# Tree-fixed count  N_G(t) = Z_G(t) / |G|^(V-1)   (V = N sites, E = 2N links).
#   S3: N_S3 = 6 [ P^N + Q^N + R^N ],  P = 1+3t^2+2t^3, Q = 1-3t^2+2t^3, R = 1-t^3
#   Z2: N_Z2 = 2 [ (1+t^2)^N + (1-t^2)^N ]
#   Z3: N_Z3 = 3 [ (1+2t^3)^N + 2 (1-t^3)^N ]
#   1 : N_1  = 1
# Total weight(M) = coefficient of t^M in N_S3.
# Burnside: orbits(t) = (1/6) [ N_S3 + 3 N_Z2 + 2 N_Z3 ].
# By holonomy group (weight w = 6/|centraliser|):
#   w=1: N_1 ; w=3: N_Z2 - N_1 ; w=2: (N_Z3 - N_1)/2 ; w=6: (N_S3 - 3(N_Z2-N_1) - (N_Z3-N_1) - N_1)/6
import sys
from itertools import product

def pmul(a, b):
    r = [0] * (len(a) + len(b) - 1)
    for i, x in enumerate(a):
        if x:
            for j, y in enumerate(b):
                r[i + j] += x * y
    return r

def ppow(a, n):
    r = [1]
    for _ in range(n):
        r = pmul(r, a)
    return r

def padd(*ps):
    n = max(len(p) for p in ps)
    return [sum(p[i] if i < len(p) else 0 for p in ps) for i in range(n)]

def pscale(p, k):
    return [k * x for x in p]

def pdiv(p, k):
    for x in p:
        assert x % k == 0
    return [x // k for x in p]

def predict(N):
    P = ppow([1, 0, 3, 2], N)
    Q = ppow([1, 0, -3, 2], N)
    R = ppow([1, 0, 0, -1], N)
    A = ppow([1, 0, 1], N)
    B = ppow([1, 0, -1], N)
    C = ppow([1, 0, 0, 2], N)
    n_s3 = pscale(padd(P, Q, R), 6)
    n_z2 = pscale(padd(A, B), 2)
    n_z3 = pscale(padd(C, pscale(R, 2)), 3)
    n_1 = [1]
    orbits = pdiv(padd(n_s3, pscale(n_z2, 3), pscale(n_z3, 2)), 6)
    w1 = padd(n_1, [0] * len(n_s3))
    w3 = padd(n_z2, pscale(n_1, -1), [0] * len(n_s3))
    w2 = pdiv(padd(n_z3, pscale(n_1, -1), [0] * len(n_s3)), 2)
    w6 = pdiv(padd(n_s3, pscale(n_z2, -3), pscale(n_z3, -1), pscale(n_1, 3)), 6)
    L = len(n_s3)
    for m in range(L):
        assert orbits[m] == w1[m] + w2[m] + w3[m] + w6[m]
        assert n_s3[m] == w1[m] + 2 * w2[m] + 3 * w3[m] + 6 * w6[m]
        assert min(w1[m], w2[m], w3[m], w6[m]) >= 0
    return orbits, n_s3, w1, w2, w3, w6

# ---- brute force on small tori (validates the formula; separate code) ----
PERMS = [(0, 1, 2), (1, 0, 2), (2, 1, 0), (0, 2, 1), (1, 2, 0), (2, 0, 1)]
IDX = {p: i for i, p in enumerate(PERMS)}
MUL = [[IDX[tuple(a[b[i]] for i in range(3))] for b in PERMS] for a in PERMS]
INV = [next(j for j in range(6) if MUL[i][j] == 0) for i in range(6)]
MAG = [0, 2, 2, 2, 3, 3]

def brute(LX, LY):
    V = LX * LY
    site = lambda x, y: (x % LX) + LX * (y % LY)
    # union-find over all 6^(2V) configurations is too large; use the definition directly on
    # the full configuration space only for the weight, via orbit = set of gauge images.
    E = 2 * V
    total = 6 ** E
    assert total <= 6 ** 8, "brute force limited to 8 links"
    seen = bytearray(total)
    pw = [6 ** i for i in range(E)]
    res = {}
    gauges = list(product(range(6), repeat=V))
    for code in range(total):
        if seen[code]:
            continue
        U = [(code // pw[i]) % 6 for i in range(E)]
        M = 0
        for y in range(LY):
            for x in range(LX):
                h = MUL[MUL[MUL[U[2 * site(x, y)]][U[2 * site(x + 1, y) + 1]]][INV[U[2 * site(x, y + 1)]]]][INV[U[2 * site(x, y) + 1]]]
                M += MAG[h]
        size = 0
        for g in gauges:
            c = 0
            for y in range(LY):
                for x in range(LX):
                    a = site(x, y)
                    for d, b in ((0, site(x + 1, y)), (1, site(x, y + 1))):
                        v = MUL[MUL[g[a]][U[2 * a + d]]][INV[g[b]]]
                        c += v * pw[2 * a + d]
            if not seen[c]:
                seen[c] = 1
                size += 1
        o, w = res.get(M, (0, 0))
        assert (size * 6) % (6 ** V) == 0
        res[M] = (o + 1, w + size * 6 // 6 ** V)   # weight = size / 6^(V-1)
    return res

def main():
    if len(sys.argv) > 1 and sys.argv[1] == "brute":
        for (lx, ly) in ((2, 2), (1, 2), (1, 3), (1, 4), (2, 1), (3, 1)):
            N = lx * ly
            orbits, wt, *_ = predict(N)
            b = brute(lx, ly)
            ok = all(b.get(m, (0, 0)) == (orbits[m], wt[m]) for m in range(len(wt)))
            print("BRUTE|lx=%d|ly=%d|plaquettes=%d|orbits_total=%d|match=%s|json=0" % (lx, ly, N, sum(orbits), "YES" if ok else "NO"))
            assert ok
        return
    N = 12
    orbits, wt, w1, w2, w3, w6 = predict(N)
    assert sum(wt) == 6 ** 13
    assert sum(orbits) == (6 ** 13 + 3 * 2 ** 13 + 2 * 3 ** 13) // 6
    out = []
    out.append("PREDICT-HEAD|stage=A|lattice=3x4|group=S3|plaquettes=12|method=character-theory-burnside|orbits_all=%d|weight_all=%d|json=0" % (sum(orbits), sum(wt)))
    co = cw = 0
    for m in range(len(wt)):
        co += orbits[m]
        cw += wt[m]
        out.append("PREDICT|M=%d|orbits=%d|weight=%d|orbits_w1=%d|orbits_w2=%d|orbits_w3=%d|orbits_w6=%d|cum_orbits=%d|cum_weight=%d|json=0"
                   % (m, orbits[m], wt[m], w1[m], w2[m], w3[m], w6[m], co, cw))
    sys.stdout.write("\n".join(out) + "\n")

main()
