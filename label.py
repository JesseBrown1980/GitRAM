#!/usr/bin/env python3
# Class label of any configuration, written from FORMAT.md alone (no code shared with the Rust program).
#   label.py <24 digits>              -> LABEL row
#   label.py --variants rows.hbp N    -> N lines "cfg key M": class representatives of the rows file moved
#                                        by a pseudo-random lattice symmetry and gauge transformation
# Exact integers only.
import sys

PERMS = [(0, 1, 2), (1, 0, 2), (2, 1, 0), (0, 2, 1), (1, 2, 0), (2, 0, 1)]
IDX = {p: i for i, p in enumerate(PERMS)}
MUL = [[IDX[tuple(a[b[i]] for i in range(3))] for b in PERMS] for a in PERMS]
INV = [row.index(0) for row in MUL]
MAG = [0, 2, 2, 2, 3, 3]


def site(x, y):
    return (x % 3) + 3 * (y % 4)


def lx(x, y):
    return 2 * site(x, y)


def ly(x, y):
    return 2 * site(x, y) + 1


def magnetic(u):
    m = 0
    for y in range(4):
        for x in range(3):
            h = MUL[MUL[MUL[u[lx(x, y)]][u[ly(x + 1, y)]]][INV[u[lx(x, y + 1)]]]][INV[u[ly(x, y)]]]
            m += MAG[h]
    return m


def orbit_key(u):
    g = [0] * 12
    g[site(1, 0)] = MUL[g[site(0, 0)]][u[lx(0, 0)]]
    g[site(2, 0)] = MUL[g[site(1, 0)]][u[lx(1, 0)]]
    for y in range(3):
        for x in range(3):
            g[site(x, y + 1)] = MUL[g[site(x, y)]][u[ly(x, y)]]
    free = [(site(2, 0), lx(2, 0), site(0, 0))]
    for y in (1, 2, 3):
        for x in range(3):
            free.append((site(x, y), lx(x, y), site(x + 1, y)))
    for x in range(3):
        free.append((site(x, 3), ly(x, 3), site(x, 0)))
    v = [MUL[MUL[g[a]][u[l]]][INV[g[b]]] for a, l, b in free]
    keys = []
    for c in range(6):
        k = 0
        for x in v:
            k = k * 6 + MUL[MUL[c][x]][INV[c]]
        keys.append(k)
    return min(keys), 6 // keys.count(keys[0])


def symmetry(u, rx, ry, tx, ty):
    o = [0] * 24
    for y in range(4):
        for x in range(3):
            px = ((-x if rx else x) + tx) % 3
            py = ((-y if ry else y) + ty) % 4
            if rx:
                o[lx(px - 1, py)] = INV[u[lx(x, y)]]
            else:
                o[lx(px, py)] = u[lx(x, y)]
            if ry:
                o[ly(px, py - 1)] = INV[u[ly(x, y)]]
            else:
                o[ly(px, py)] = u[ly(x, y)]
    return o


def class_label(u):
    keys = set()
    for rx in (0, 1):
        for ry in (0, 1):
            for tx in range(3):
                for ty in range(4):
                    keys.add(orbit_key(symmetry(u, rx, ry, tx, ty))[0])
    return min(keys), len(keys), orbit_key(u)[1]


def gauge(u, g):
    o = [0] * 24
    for y in range(4):
        for x in range(3):
            o[lx(x, y)] = MUL[MUL[g[site(x, y)]][u[lx(x, y)]]][INV[g[site(x + 1, y)]]]
            o[ly(x, y)] = MUL[MUL[g[site(x, y)]][u[ly(x, y)]]][INV[g[site(x, y + 1)]]]
    return o


def from_key(key):
    u = [0] * 24
    links = [lx(2, 0)] + [lx(x, y) for y in (1, 2, 3) for x in range(3)] + [ly(x, 3) for x in range(3)]
    for l in reversed(links):
        u[l] = key % 6
        key //= 6
    return u


def main():
    if sys.argv[1] == "--variants":
        n = int(sys.argv[3])
        keys = []
        with open(sys.argv[2]) as f:
            for line in f:
                if line.startswith("R|"):
                    fl = line.split("|")
                    keys.append((int(fl[2][4:]), int(fl[3][2:])))
        state = 88172645463325252
        step = max(1, len(keys) // n)
        for key, m in keys[::step][:n]:
            r = []
            for _ in range(16):
                state ^= (state << 13) & 0xFFFFFFFFFFFFFFFF
                state ^= state >> 7
                state ^= (state << 17) & 0xFFFFFFFFFFFFFFFF
                r.append(state >> 11)
            u = symmetry(from_key(key), r[0] % 2, r[1] % 2, r[2] % 3, r[3] % 4)
            u = gauge(u, [x % 6 for x in r[4:16]])
            assert class_label(u)[0] == key and magnetic(u) == m
            print("".join(str(x) for x in u), key, m)
        return
    u = [int(c) for c in sys.argv[1] if c.isdigit()]
    assert len(u) == 24 and max(u) < 6
    m = magnetic(u)
    key, size, w = class_label(u)
    core = lambda m0: 1 if 0 < m <= m0 else 0
    print("LABEL|class_key=%d|M=%d|size=%d|w=%d|W=%d|flat=%d|core_m11=%d|core_m12=%d|core_m13=%d|json=0"
          % (key, m, size, w, size * w, 1 if m == 0 else 0, core(11), core(12), core(13)))


main()
