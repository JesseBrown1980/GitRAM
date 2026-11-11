#!/usr/bin/env python3
"""Exhaustive Stage B comparison of independent Stage A rows and engine rows.

Engine C/T transport is emitted by a nonmathematical instrumentation of the
existing pinned engine cert_gate.py. Stage A source and results are immutable
inputs, with rows reconstructed from the clean-room commit due to a disclosed
gzip archive corruption. Cross-map uses unchanged Stage A Rust lat.rs.

Exact integers, streamed engine rows, never reads r or launches at any coupling.
Any mismatch prints the first witness and exits nonzero. SHA-256 comparisons
are integrity gates; graph equality is per-row, not inferred from fingerprints.
"""
import argparse
import collections
import functools
import gzip
import hashlib
import pathlib
import sys


def pairs(s):
    if s == "-":
        return ()
    result = []
    for item in s.split(","):
        if not item:
            raise ValueError("empty list entry")
        k, sep, v = item.partition(":")
        if not sep or int(v) <= 0:
            raise ValueError("malformed pair")
        result.append((int(k), int(v)))
    if len(set(k for k, _ in result)) != len(result):
        raise ValueError("duplicate pair")
    return tuple(result)


def hbp(s, tag):
    parts = s.rstrip("\n").split("|")
    if not parts or parts[0] != tag or parts[-1] != "json=0":
        raise ValueError("expected "+tag+" record")
    a = {}
    for entry in parts[1:-1]:
        k, sep, v = entry.partition("=")
        if not sep or k in a:
            raise ValueError("duplicate/malformed field: "+entry)
        a[k] = v
    return a


def sha(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while True:
            b=f.read(1 << 20)
            if not b:
                break
            h.update(b)
    return h.hexdigest()


def mismatch(field, engine, label, want, got):
    print("STAGE_B|MISMATCH|field=%s|engine_key=%s|stageA_key=%s|expected=%s|observed=%s" %
          (field, engine, label, str(want)[:500], str(got)[:500]), flush=True)
    raise SystemExit(2)


def load_stage_a(path, m0):
    tails = []
    rows = []
    labels = {}
    with open(path, "rt", encoding="ascii", newline="\n") as inp:
        for raw in inp:
            typ = raw.partition("|")[0]
            if typ == "HEAD":
                hdr=hbp(raw,"HEAD")
                if int(hdr["m0"]) != m0:
                    raise ValueError("Stage A wrong cutoff")
                want_classes=int(hdr["classes"])
                want_tails=int(hdr["tailsets"])
            elif typ=="T":
                a=hbp(raw,"T")
                if int(a["t"]) != len(tails):
                    raise ValueError("nonsequential Stage A tail id")
                back=a["back"]
                if int(a["n"]) != sum(c for _,c in pairs(back)):
                    raise ValueError("bad Stage A tail back count")
                tails.append((int(a["M"]), back))
            elif typ=="R":
                a=hbp(raw,"R")
                i=len(rows)
                if int(a["id"]) != i:
                    raise ValueError("nonsequential Stage A class id")
                k=int(a["key"])
                if k in labels:
                    raise ValueError("duplicate Stage A class key")
                row=(k,int(a["M"]),int(a["size"]),int(a["W"]),int(a["flat"]),a["core"],a["tail"])
                if row[4]+sum(c for _,c in pairs(row[5]))+sum(c for _,c in pairs(row[6]))!=72:
                    raise ValueError("Stage A row move total !=72")
                labels[k]=i
                rows.append(row)
            else:
                raise ValueError("unknown Stage A row tag "+typ)
    if len(rows)!=want_classes or len(tails)!=want_tails:
        raise ValueError("Stage A header mismatch")
    print("STAGE_A_INPUT|m0=%d|classes=%d|tailsets=%d|sha256=%s|PASS" %
          (m0,len(rows),len(tails),sha(path)),flush=True)
    return rows,tails,labels


def load_map(path, rows, labels):
    eng_to_id={}
    seen=set()
    for raw in pathlib.Path(path).read_text(encoding="ascii").splitlines():
        a=hbp(raw,"B")
        engine=int(a["engine"])
        stage=int(a["stage_key"])
        if engine in eng_to_id:
            raise ValueError("duplicate engine key in class map")
        if stage not in labels:
            mismatch("class_lookup_missing",engine,stage,"known Stage A label","absent")
        sid=labels[stage]
        if sid in seen:
            mismatch("class_bijection_not_injective",engine,stage,"unused label",sid)
        seen.add(sid)
        row=rows[sid]
        if (int(a["M"]),int(a["size"]),int(a["W"]))!=row[1:4]:
            mismatch("lookup_metadata",engine,stage,row[1:4],(a["M"],a["size"],a["W"]))
        eng_to_id[engine]=sid
    if len(seen)!=len(rows):
        raise ValueError("class map not onto: unseen Stage A classes=%d" % (len(rows)-len(seen)))
    print("STAGE_B_MAP|classes=%d|injective=PASS|surjective=PASS|magnetic=PASS|source_derived_metadata=CORROBORATED" %
          len(rows),flush=True)
    return eng_to_id


def load_engine_meta(path, eng_to_id, rows):
    bykey={}
    with open(path,"rt",encoding="ascii") as f:
        for raw in f:
            a=hbp(raw,"V")
            k=int(a["key"])
            if k in bykey or k not in eng_to_id:
                raise ValueError("unknown or duplicate engine metadata key")
            row=rows[eng_to_id[k]]
            obs=(int(a["M"]),int(a["size"]),int(a["W"]))
            if obs!=row[1:4]:
                mismatch("engine_source_derived_metadata",k,row[0],row[1:4],obs)
            bykey[k]=obs
    if len(bykey)!=len(rows):
        raise ValueError("missing engine metadata records")
    print("ENGINE_SOURCE_METADATA|classes=%d|size=CORROBORATED|weight=CORROBORATED|independence=SUPPORTING_ONLY" %
          len(rows),flush=True)


def compare(args):
    m0=args.m0
    rows,tails,labels=load_stage_a(args.stagea,m0)
    if sha(args.stagea)!=args.stagea_sha256:
        raise ValueError("Stage A reconstructed row SHA-256 unexpected")
    eng_to_id=load_map(args.mapping,rows,labels)
    load_engine_meta(args.meta,eng_to_id,rows)

    @functools.lru_cache(maxsize=16384)
    def tail_token(tid):
        m,back=tails[tid]
        bs=[]
        for idx,c in pairs(back):
            if idx<0 or idx>=len(rows):
                raise ValueError("Stage A tail back id out of range")
            bs.extend([idx]*c)
        return (m,tuple(bs))

    count=0
    total_tail=total_back=total_core=0
    current=None
    engine_row_sha=hashlib.sha256()

    def finish():
        nonlocal count,total_tail,total_back,total_core,current
        if current is None:
            return
        key, m, flat, core, observed_tails=current
        sid=eng_to_id[key]
        row=rows[sid]
        label=row[0]
        if (m,flat)!=(row[1],row[4]):
            mismatch("M_or_flat",key,label,(row[1],row[4]),(m,flat))
        expected_core=collections.Counter(dict(pairs(row[5])))
        if core!=expected_core:
            mismatch("core_transitions",key,label,expected_core,core)
        expected_tails=collections.Counter()
        for tid,mult in pairs(row[6]):
            expected_tails[tail_token(tid)] += mult
        if observed_tails!=expected_tails:
            missing=expected_tails-observed_tails
            extra=observed_tails-expected_tails
            mismatch("tail_back_multisets",key,label,list(missing.items())[:2],list(extra.items())[:2])
        obs_moves=flat+sum(core.values())+sum(observed_tails.values())
        if obs_moves!=72:
            mismatch("row_move_conservation",key,label,72,obs_moves)
        count+=1
        total_tail+=sum(observed_tails.values())
        total_back+=sum(c*len(tok[1]) for tok,c in observed_tails.items())
        total_core+=len(core)
        if count%10000==0:
            print("STAGE_B_PROGRESS|m0=%d|rows_checked=%d" % (m0,count),flush=True)

    with gzip.open(args.engine,"rb") as f:
        for raw in f:
            engine_row_sha.update(raw)
            line=raw.decode("ascii").rstrip("\n")
            if line.startswith("floorcert-rows-v1 m0="):
                if line!=f"floorcert-rows-v1 m0={m0}" or current is not None:
                    raise ValueError("bad engine row header")
                continue
            if line.startswith("C "):
                finish()
                fields=line.split()
                key=int(fields[1])
                if key not in eng_to_id:
                    mismatch("engine_class_not_in_mapping",key,"?", "mapped class","unmapped")
                sid=eng_to_id[key]
                core=collections.Counter()
                for pair in fields[4:]:
                    ek,c=pair.split(":")
                    ek=int(ek)
                    if ek not in eng_to_id:
                        mismatch("unmapped_core_target",key,rows[sid][0],ek,"absent")
                    core[eng_to_id[ek]]+=int(c)
                current=(key,int(fields[2]),int(fields[3]),core,collections.Counter())
            elif line.startswith("T "):
                if current is None:
                    raise ValueError("tail before class row")
                fields=line.split()
                M=int(fields[1])
                hist=list(map(int,fields[2].split(",")))
                if len(hist)!=9:
                    raise ValueError("engine 9-bin tail histogram absent")
                back=[]
                for ek in fields[3:]:
                    k=int(ek)
                    if k not in eng_to_id:
                        mismatch("tail_back_unmapped",current[0],"?",k,"unmapped")
                    back.append(eng_to_id[k])
                if sum(hist)+len(back)!=72:
                    mismatch("tail_second_move_conservation",current[0],rows[eng_to_id[current[0]]][0],72,sum(hist)+len(back))
                current[4][(M,tuple(sorted(back)))]+=1
            elif line.startswith("END "):
                finish()
                current=None
                fields=line.split()
                if tuple(map(int,fields[1:]))!=(count,total_tail,total_back):
                    mismatch("engine_end_totals","END","ALL",(count,total_tail,total_back),fields[1:])
            else:
                raise ValueError("unexpected engine row line "+line[:100])
    calculated=engine_row_sha.hexdigest()
    if calculated!=args.engine_sha256:
        raise ValueError("engine row byte SHA mismatch "+calculated)
    if current is not None or count!=len(rows):
        raise ValueError("engine stream missing END or classes: got %d want %d"%(count,len(rows)))
    print("STAGE_B|PASS|m0=%d|bijection=%d_to_%d|rows_compared=%d|core_nonzero=%d|tail_entries=%d|back_multiset_total=%d|every_transition=EQUAL|every_tail=EQUAL|engine_row_sha256=%s|stageA_row_sha256=%s|class_metadata=SOURCE_DERIVED_CORROBORATION|fingerprint=PRELIMINARY_ONLY|r9_executions=0" %
          (m0,len(rows),len(rows),count,total_core,total_tail,total_back,calculated,args.stagea_sha256),flush=True)


def main():
    p=argparse.ArgumentParser()
    p.add_argument("--m0",type=int,choices=(11,12,13),required=True)
    for arg in ("stagea","mapping","meta","engine"):
        p.add_argument("--"+arg,required=True)
    for arg in ("stagea_sha256","engine_sha256"):
        p.add_argument("--"+arg.replace("_","-"),required=True,dest=arg)
    a=p.parse_args()
    compare(a)

if __name__=="__main__":
    try:
        main()
    except Exception as exc:
        print("STAGE_B|HOLD|exception_type=%s|message=%s" % (type(exc).__name__,str(exc)[:500]),flush=True)
        raise
