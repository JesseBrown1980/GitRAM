#!/usr/bin/env python3
# Second reading of a rows file, in another language, sharing no code with the Rust program:
# file SHA-256, section-3 totals, per-M table, class-size histogram, class-level detailed balance,
# and the label-free fingerprint (hashlib).  Exact integers only.
import hashlib
import resource
import sys


def pairs(s):
    if s == "-":
        return []
    out = []
    for item in s.split(","):
        a, c = item.split(":")
        out.append((int(a), int(c)))
    return out


def main():
    path = sys.argv[1]
    h = hashlib.sha256()
    tail_m = []
    tail_back = []
    rows = []
    with open(path, "rb") as f:
        for raw in f:
            h.update(raw)
            line = raw.decode("ascii").rstrip("\n")
            fl = line.split("|")
            assert fl[-1] == "json=0"
            kv = dict(x.split("=", 1) for x in fl[1:-1])
            if fl[0] == "HEAD":
                m0 = int(kv["m0"])
                nclass = int(kv["classes"])
                ntail = int(kv["tailsets"])
            elif fl[0] == "T":
                assert int(kv["t"]) == len(tail_m)
                tail_m.append(int(kv["M"]))
                tail_back.append(kv["back"])
            elif fl[0] == "R":
                assert int(kv["id"]) == len(rows)
                rows.append((int(kv["M"]), int(kv["size"]), int(kv["W"]), int(kv["flat"]), kv["core"], kv["tail"], int(kv["key"]), int(kv["w"])))
            else:
                raise SystemExit("unknown row tag")
    assert len(rows) == nclass and len(tail_m) == ntail
    # ordering of the labelling: (M, key) strictly increasing
    for i in range(1, nclass):
        assert (rows[i - 1][0], rows[i - 1][6]) < (rows[i][0], rows[i][6])
    per_m = {}
    hist = {}
    nz = flat_sum = tail_entries = back_total = 0
    bal_bad = bal_pairs = 0
    core_cache = [None] * nclass
    for i, r in enumerate(rows):
        core_cache[i] = dict(pairs(r[4]))
    for i, (m, size, wt, flat, core, tail, key, w) in enumerate(rows):
        assert wt == size * w and 0 < m <= m0
        c, o, ww = per_m.get(m, (0, 0, 0))
        per_m[m] = (c + 1, o + size, ww + wt)
        hist[size] = hist.get(size, 0) + 1
        flat_sum += flat
        cp = core_cache[i]
        nz += len(cp)
        moves = flat + sum(cp.values())
        for b, cnt in cp.items():
            bal_pairs += 1
            if wt * cnt != rows[b][2] * core_cache[b].get(i, 0):
                bal_bad += 1
        for t, mult in pairs(tail):
            assert tail_m[t] > m0
            moves += mult
            tail_entries += mult
            back_total += mult * sum(c2 for _, c2 in pairs(tail_back[t]))
        assert moves == 72
    out = []
    cc = co = cw = 0
    for m in sorted(per_m):
        c, o, ww = per_m[m]
        cc += c
        co += o
        cw += ww
        out.append("VPERM|M=%d|classes=%d|orbits=%d|weight=%d|cum_classes=%d|cum_orbits=%d|cum_weight=%d|json=0" % (m, c, o, ww, cc, co, cw))
    for s in sorted(hist):
        out.append("VHIST|class_size=%d|classes=%d|json=0" % (s, hist[s]))
    out.append("VTOTALS|m0=%d|core_classes=%d|core_core_nonzero=%d|flat_count_sum=%d|tail_entries=%d|back_multiset_total=%d|tail_sets_distinct=%d|json=0"
               % (m0, nclass, nz, flat_sum, tail_entries, back_total, ntail))
    out.append("VBALANCE|pairs=%d|violations=%d|json=0" % (bal_pairs, bal_bad))
    out.append("VROWS|sha256=%s|json=0" % h.hexdigest())
    col = ["%d,%d,%d,%d" % (r[0], r[1], r[2], r[3]) for r in rows]
    for _ in range(3):
        new = []
        for i, r in enumerate(rows):
            core = sorted("%s:%d" % (col[b], c) for b, c in core_cache[i].items())
            tl = []
            for t, mult in pairs(r[5]):
                back = []
                for b, c in pairs(tail_back[t]):
                    back.extend([col[b]] * c)
                back.sort()
                s = "%d:%s" % (tail_m[t], ".".join(back))
                tl.extend([s] * mult)
            tl.sort()
            text = col[i] + "|" + ",".join(core) + "|" + ",".join(tl)
            new.append(hashlib.sha256(text.encode("utf-8")).hexdigest())
        col = new
    col.sort()
    fp = hashlib.sha256("\n".join(col).encode("utf-8")).hexdigest()
    out.append("VFINGERPRINT|m0=%d|rounds=3|sha256=%s|distinct_colour3=%d|classes=%d|json=0" % (m0, fp, len(set(col)), nclass))
    out.append("VRUN|maxrss_kib=%d|json=0" % resource.getrusage(resource.RUSAGE_SELF).ru_maxrss)
    sys.stdout.write("\n".join(out) + "\n")


main()
