#!/usr/bin/env python3
"""Stage B *engine-side transport only*, NOT a Hamiltonian independence checker.

Reference: published 3x4 S3 floorcert engine at immutable commit
e74d8a46e86034096fd9c9231c87f594f17a93bf
No Stage A code or expected result is read. Never runs at any coupling.

Usage:
    python3 -I -B engine_key_decoder.py --selftest
    python3 -I -B engine_key_decoder.py --one 1
    python3 -I -B engine_key_decoder.py --keys-file core_keys_m11.bin --m0 11 \\
        --pin-sha256 <64hex> --output output.hbp

Output format (one record per class):
    E|key=<decimal>|links=<24 comma-delimited triples>
Link slots: h(x,y), x=0..2, y=0..3; then v(x,y), same order.
Triple abc is a permutation 0->a,1->b,2->c, no group-operation convention needed.
"""
import argparse
import hashlib
import pathlib
import sys

PERMUTATIONS = ("012", "021", "102", "120", "201", "210")
NUM_LINKS = 24
BASE = 6
FREE_LINKS = (8, 9, 10, 11, 15, 16, 17, 18, 19, 20, 21, 22, 23)
TREE_LINKS = tuple(i for i in range(NUM_LINKS) if i not in FREE_LINKS)
MAX_KEY = BASE ** len(FREE_LINKS)
LIMIT_KIB = 2097152
assert NUM_LINKS == 24 and len(FREE_LINKS) == 13 and len(TREE_LINKS) == 11
assert TREE_LINKS == (0, 1, 2, 3, 4, 5, 6, 7, 12, 13, 14)


def decode_key(key):
    if type(key) is not int or key < 0 or key >= MAX_KEY:
        raise ValueError("engine class key out of range")
    digits = [0] * len(FREE_LINKS)
    remaining = key
    for i in range(len(digits) - 1, -1, -1):
        remaining, digits[i] = divmod(remaining, BASE)
    assert remaining == 0
    links = [0] * NUM_LINKS
    for slot, value in zip(FREE_LINKS, digits):
        links[slot] = value
    return tuple(links)


def encode_key(links):
    if len(links) != NUM_LINKS:
        raise ValueError("expected exactly 24 oriented links")
    if any(type(a) is not int or a < 0 or a >= BASE for a in links):
        raise ValueError("invalid permutation code")
    if any(links[i] != 0 for i in TREE_LINKS):
        raise ValueError("spanning-tree links must be identity")
    key = 0
    for slot in FREE_LINKS:
        key = BASE * key + links[slot]
    return key


def permutation_word(links):
    return ",".join(PERMUTATIONS[a] for a in links)


def record(key):
    return "E|key=%d|links=%s\n" % (key, permutation_word(decode_key(key)))


# The magnetic invariant below only checks that the transported configuration
# falls within the claimed cutoff. It must not be represented as independent
# full-row checking.
def inverse(p):
    out = [0, 0, 0]
    for i in range(3):
        out[p[i]] = i
    return tuple(out)


PERMS = [tuple(int(c) for c in p) for p in PERMUTATIONS]
INVERSE = tuple(PERMS.index(inverse(p)) for p in PERMS)
COST = tuple(3 - sum(1 for i in range(3) if p[i] == i) for p in PERMS)
MULT = tuple(tuple(PERMS.index(tuple(PERMS[a][PERMS[b][i]] for i in range(3))) for b in range(6)) for a in range(6))
assert COST == (0, 2, 2, 3, 3, 2)


def magnetic(links):
    total = 0
    for x in range(3):
        for y in range(4):
            h0 = 4 * x + y
            vright = 12 + 4 * ((x + 1) % 3) + y
            htop = 4 * x + (y + 1) % 4
            vleft = 12 + 4 * x + y
            a = MULT[links[h0]][links[vright]]
            b = MULT[a][INVERSE[links[htop]]]
            h = MULT[b][INVERSE[links[vleft]]]
            total += COST[h]
    return total


def key_file_records(path):
    # Generator: constant memory even for large key files.
    prev = -1
    with open(path, "rb") as fh:
        while True:
            octets = fh.read(8)
            if not octets:
                break
            if len(octets) != 8:
                raise ValueError("truncated 64-bit key")
            key = int.from_bytes(octets, "little")
            if key <= prev:
                raise ValueError("duplicate or nonincreasing class key")
            if key >= MAX_KEY:
                raise ValueError("key outside 6^13")
            prev = key
            yield key


def hash_file(path):
    digest = hashlib.sha256()
    with open(path, "rb") as fh:
        while True:
            chunk = fh.read(1 << 20)
            if not chunk:
                break
            digest.update(chunk)
    return digest.hexdigest()


def run_selftest():
    assert decode_key(0) == (0,) * NUM_LINKS
    assert encode_key(decode_key(MAX_KEY - 1)) == MAX_KEY - 1
    assert decode_key(1)[23] == 1 and magnetic(decode_key(1)) == 4
    assert magnetic(decode_key(3)) == 6
    for key in (0, 1, 5, 6, 35, 36, 123456, MAX_KEY - 2, MAX_KEY - 1):
        assert encode_key(decode_key(key)) == key
        assert len(permutation_word(decode_key(key)).split(",")) == NUM_LINKS
    for invalid in (-1, MAX_KEY):
        try:
            decode_key(invalid)
        except ValueError:
            pass
        else:
            raise AssertionError("invalid key accepted")
    bad = list(decode_key(1))
    bad[0] = 1
    try:
        encode_key(bad)
    except ValueError:
        pass
    else:
        raise AssertionError("nonidentity tree link accepted")
    assert record(0) == "E|key=0|links=" + ",".join(["012"] * 24) + "\n"
    print("DECODER_SELFTEST|PASS|coupling=none|algorithm=engine_side_transport")


def main():
    p = argparse.ArgumentParser()
    mode = p.add_mutually_exclusive_group(required=True)
    mode.add_argument("--selftest", action="store_true")
    mode.add_argument("--one", type=int)
    mode.add_argument("--keys-file")
    p.add_argument("--m0", type=int, choices=(11, 12, 13))
    p.add_argument("--pin-sha256")
    p.add_argument("--output")
    args = p.parse_args()

    if args.selftest:
        if any(x is not None for x in (args.m0, args.pin_sha256, args.output)):
            p.error("selftest takes no other arguments")
        run_selftest()
        return
    if args.one is not None:
        if any(x is not None for x in (args.m0, args.pin_sha256, args.output)):
            p.error("--one takes no other arguments")
        print(record(args.one), end="")
        return

    if args.m0 is None or args.pin_sha256 is None or args.output is None:
        p.error("--keys-file requires --m0, --pin-sha256 and --output")
    expected = args.pin_sha256
    if len(expected) != 64 or any(c not in "0123456789abcdef" for c in expected):
        p.error("--pin-sha256 must be 64 lowercase hex characters")
    keyfile = pathlib.Path(args.keys_file)
    output = pathlib.Path(args.output)
    if not keyfile.is_file():
        raise ValueError("key file does not exist")
    if keyfile.resolve() == output.resolve():
        raise ValueError("output cannot overwrite input")
    observed = hash_file(keyfile)
    if observed != expected:
        raise ValueError("KEY_SHA256_MISMATCH")
    output.parent.mkdir(parents=True, exist_ok=True)

    count = 0
    with output.open("w", encoding="ascii", newline="\n") as fh:
        for key in key_file_records(keyfile):
            m = magnetic(decode_key(key))
            if not (0 < m <= args.m0):
                raise ValueError("invalid magnetic cutoff in class key")
            fh.write(record(key))
            count += 1

    print("ENGINE_DECODER|PASS|m0=%d|classes=%d|keys_sha256=%s|transport_sha256=%s|coupling=none" %
          (args.m0, count, observed, hash_file(output)))


if __name__ == "__main__":
    try:
        main()
    except (ValueError, OSError, AssertionError) as exc:
        print("ENGINE_DECODER|FAIL|reason=%s" % str(exc), file=sys.stderr)
        sys.exit(1)
