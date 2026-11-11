#!/usr/bin/env python3
"""Instrument a COPY of the immutable v1 engine verifier, for Stage B transport.

The mathematical computation and canonical row hash are unchanged. Added
side effects write the exact existing C/T row stream to gzip and per-class
orbit-stabilizer metadata to a text file. Both files have no coupling value.

The new source is auditable as a diff; it is NOT substituted for the required
pinned mathematical verifier in any r=9 certificate workflow.
"""
import hashlib
import pathlib
import sys

if len(sys.argv) != 3:
    raise SystemExit("usage: patch_engine_export.py ORIGINAL_CERT_GATE.py COPY_OUT.py")
src = pathlib.Path(sys.argv[1])
dst = pathlib.Path(sys.argv[2])
assert src.is_file() and src.resolve() != dst.resolve()
assert not dst.exists(), "do not overwrite an existing verifier"
lines = src.read_text(encoding="utf-8").splitlines(keepends=True)
out = []
counts = {x: 0 for x in ("cls_size", "size_push", "gz_init", "gz_feed", "metadata", "gz_close")}
for line in lines:
    if line.startswith("cls_W = []"):
        out.append(line)
        out.append("cls_size = []   # Stage B transport: class size from the same orbit-stabilizer enumeration\n")
        counts["cls_size"] += 1
    elif line.strip() == "cls_W.append((mw % 8) * len(imgs))":
        out.append(line)
        out.append("    cls_size.append(len(imgs))\n")
        counts["size_push"] += 1
    elif line == "rh = hashlib.sha256()\n":
        out.append("import gzip\n")
        out.append("import os\n")
        out.append("_stageb_gz = gzip.open(os.environ['STAGEB_EXPORT_PATH'], 'wt', encoding='ascii', newline='\\n', compresslevel=1)\n")
        out.append("_stageb_meta = open(os.environ['STAGEB_METADATA_PATH'], 'wt', encoding='ascii', newline='\\n')\n")
        out.append(line)
        counts["gz_init"] += 1
    elif line == "    nbytes += len(b)\n":
        out.append(line)
        out.append("    _stageb_gz.write(s)\n")
        counts["gz_feed"] += 1
    elif line == "    ntail += len(tails)\n":
        out.append(line)
        out.append("    _stageb_meta.write('V|key=%d|M=%d|size=%d|W=%d|json=0\\n' % (cls2cert[ci], M, cls_size[ci], cls_W[ci]))\n")
        counts["metadata"] += 1
    elif line == "own_hash = rh.hexdigest()\n":
        out.append(line)
        out.append("_stageb_gz.close()\n")
        out.append("_stageb_meta.close()\n")
        counts["gz_close"] += 1
    else:
        out.append(line)
assert all(v == 1 for v in counts.values()), "unexpected source change; refusing patch "+str(counts)
dst.parent.mkdir(parents=True, exist_ok=True)
dst.write_text("".join(out), encoding="utf-8", newline="\n")
compile("".join(out), str(dst), "exec")
sha=lambda p: hashlib.sha256(p.read_bytes()).hexdigest()
print("ENGINE_EXPORT_PATCH|PASS|source_sha256="+sha(src)+"|patched_sha256="+sha(dst)+"|additions=7|original_unchanged=1|coupling=none")
