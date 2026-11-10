#!/usr/bin/env python3
"""Engine-side Stage B convention preflight, not clean-room code.

Tests explicit 0->a permutation words, orientation and multiplication on
hand-built physical configurations, and a generic target encoding translator.
The actual Stage A lookup convention is not available yet; its end-to-end
test is a separate required gate. No coupling value is used.
"""
import sys
import importlib.util
from pathlib import Path
# -I excludes the script directory from sys.path. Load the pinned adjacent file
# explicitly, rather than disabling isolated interpreter mode.
_adapter_path = Path(__file__).resolve().with_name("engine_key_decoder.py")
_spec = importlib.util.spec_from_file_location("_stageb_engine_transport", _adapter_path)
if _spec is None or _spec.loader is None:
    raise RuntimeError("cannot locate the frozen engine decoder")
_adapter = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(_adapter)
decode_key = _adapter.decode_key
encode_key = _adapter.encode_key
record = _adapter.record
magnetic = _adapter.magnetic
PERMUTATIONS = _adapter.PERMUTATIONS

ALL_LINKS = tuple(("h" if k < 12 else "v",
                   (k if k < 12 else k - 12) // 4,
                   (k if k < 12 else k - 12) % 4) for k in range(24))
LINK_SET = set(ALL_LINKS)
VALID_WORDS = set(PERMUTATIONS)
assert len(LINK_SET) == 24


def inverse(p):
    r = [None] * 3
    for i, j in enumerate(p):
        r[j] = i
    return tuple(r)


def multiply(a, b):
    """a after b, with tuples describing i->image[i]."""
    return tuple(a[b[i]] for i in range(3))


def parse_word(s):
    if s not in VALID_WORDS:
        raise ValueError("word must be a permutation of 012")
    return tuple(map(int, s))


def unpack_engine_line(line):
    parts = line.rstrip("\n").split("|")
    if len(parts) != 3 or parts[0] != "E" or not parts[1].startswith("key=") or not parts[2].startswith("links="):
        raise ValueError("malformed engine transport")
    key = int(parts[1][4:])
    words = parts[2][6:].split(",")
    if len(words) != 24:
        raise ValueError("wrong number of links")
    codes = tuple(PERMUTATIONS.index(w) for w in words)
    if codes != decode_key(key):
        raise ValueError("permutation/engine-key mismatch")
    return dict(zip(ALL_LINKS, map(parse_word, words)))


def to_target(physical, order, convention):
    """Target entry is (kind,x,y,reversed_edge); convention=image or preimage."""
    if convention not in ("image", "preimage") or len(order) != 24:
        raise ValueError("bad target convention or count")
    if set((k, x, y) for k, x, y, rev in order) != LINK_SET:
        raise ValueError("target links not bijective")
    result = []
    for k, x, y, rev in order:
        if type(rev) is not bool:
            raise ValueError("bad reversed edge flag")
        p = physical[(k, x, y)]
        if rev:
            p = inverse(p)
        if convention == "preimage":
            p = inverse(p)
        result.append("".join(map(str, p)))
    return tuple(result)


def from_target(words, order, convention):
    if len(words) != 24 or len(order) != 24 or convention not in ("image", "preimage"):
        raise ValueError("bad target schema")
    result = {}
    for word, (k, x, y, rev) in zip(words, order):
        p = parse_word(word)
        if convention == "preimage":
            p = inverse(p)
        if rev:
            p = inverse(p)
        if (k, x, y) in result:
            raise ValueError("duplicate target link")
        result[(k, x, y)] = p
    if set(result) != LINK_SET:
        raise ValueError("incomplete target")
    return result


def holonomy(field, x, y):
    x %= 3
    y %= 4
    paths = (field[("h", x, y)], field[("v", (x + 1) % 3, y)],
             inverse(field[("h", x, (y + 1) % 4)]),
             inverse(field[("v", x, y)]))
    h = (0, 1, 2)
    for p in paths:
        h = multiply(h, p)
    return "".join(map(str, h))


def full_m(field):
    return sum(3 - sum(holonomy(field, x, y)[i] == str(i) for i in range(3))
               for x in range(3) for y in range(4))


def run():
    # The final free link is v(2,3). A single transposition enters two
    # plaquettes, each with cost 2, and a single 3-cycle has cost 3 twice.
    trans = unpack_engine_line(record(1))
    assert trans[("v", 2, 3)] == (0, 2, 1)
    assert (holonomy(trans, 1, 3), holonomy(trans, 2, 3)) == ("021", "021")
    assert full_m(trans) == magnetic(decode_key(1)) == 4
    print("KNOWN|one_transposition|M=4|both_plaquettes=021|PASS")

    cycle = unpack_engine_line(record(3))
    assert cycle[("v", 2, 3)] == (1, 2, 0)
    assert (holonomy(cycle, 1, 3), holonomy(cycle, 2, 3)) == ("120", "201")
    assert full_m(cycle) == magnetic(decode_key(3)) == 6
    print("KNOWN|one_3cycle|M=6|positive=120|reversed=201|PASS")

    # A noncommuting pair catches composition order even when M is unchanged.
    # h(2,3)=102, v(0,3)=120 -> h*v=021, but v*h=210.
    links = [0] * 24
    links[11], links[15] = 2, 3
    key = encode_key(links)
    assert key == 25194240
    noncomm = unpack_engine_line(record(key))
    assert holonomy(noncomm, 2, 3) == "021"
    assert holonomy(noncomm, 2, 2) == "102"
    assert holonomy(noncomm, 0, 3) == "201"
    assert multiply(parse_word("120"), parse_word("102")) == parse_word("210")
    assert full_m(noncomm) == magnetic(decode_key(key)) == 7
    print("KNOWN|noncommuting_102_then_120|M=7|plaquette23=021|PASS")

    # The target side may choose any link order, orientation, or word meaning.
    # These synthetic choices test the translator without seeing Stage A.
    tests = (
        ("reverse_slots", tuple((*k, False) for k in reversed(ALL_LINKS)), "image"),
        ("reverse_edges", tuple((*k, True) for k in ALL_LINKS), "image"),
        ("preimage", tuple((*k, False) for k in ALL_LINKS), "preimage"),
        ("combined", tuple((*k, i % 3 == 0) for i, k in enumerate(reversed(ALL_LINKS))), "preimage"),
    )
    for label, order, encoding in tests:
        for k, field in ((1, trans), (3, cycle), (key, noncomm)):
            back = from_target(to_target(field, order, encoding), order, encoding)
            assert back == field
            assert full_m(back) == magnetic(decode_key(k))
            assert all(holonomy(back, x, y) == holonomy(field, x, y)
                       for x in range(3) for y in range(4))
        print("TRANSLATION|%s|three_roundtrips=PASS" % label)

    # Wrong decoding of the same triplets can preserve M but invert the
    # *oriented* holonomy; both the word and composition controls catch it.
    order = tuple((*k, False) for k in ALL_LINKS)
    wrong = from_target(to_target(cycle, order, "preimage"), order, "image")
    assert wrong != cycle and holonomy(wrong, 1, 3) == "201"
    assert multiply(parse_word("120"), parse_word("102")) != parse_word(holonomy(noncomm, 2, 3))
    print("FAULT|preimage_read_as_image|oriented_holonomy_detected|PASS")
    print("FAULT|reverse_product|oriented_holonomy_detected|PASS")

    try:
        unpack_engine_line(record(3).replace("120", "201", 1))
    except ValueError:
        print("FAULT|tampered_transport_word|rejected|PASS")
    else:
        raise AssertionError("tampered permutation accepted")

    print("CONVENTION_PREFLIGHT|PASS|independent_stage_A_lookup=NOT_YET_TESTED|coupling=none")


if __name__ == "__main__":
    try:
        run()
    except (ValueError, AssertionError) as exc:
        print("CONVENTION_PREFLIGHT|FAIL|reason=%s" % exc, file=sys.stderr)
        raise
