#!/usr/bin/env python3
"""Full Stage B comparison: all mapped class rows and every tail back multiset.

Exactly two immutable, independently written implementations:
 - Engine rows: original Rust engine, with read-only instrumentation to export its data
 - Stage A rows/lookup: clean-room second commit, unmodified
 This comparator is not a third independent reconstruction.

Inputs:
  stageb_compare.py STAGE_A_ROWS.hbp ENGINE_ROWS.hbp MAP.hbp M0
No coupling input exists. All comparisons exact; first mismatch => exit 1.
"""
from collections import Counter
from hashlib import sha256
import sys

def fields(line, tag):
    a=line.rstrip("\n").split("|")
    if a[0] != tag:
        raise ValueError("expected tag %s got %s" % (tag, a[0]))
    d={}
    for part in a[1:]:
        if "=" not in part:
            raise ValueError("bad field "+part)
        k,v=part.split("=",1)
        if k in d:
            raise ValueError("duplicate field "+k)
        d[k]=v
    return d

def pairs(s):
    if s in ("", "-"):
        return ()
    z=[]
    seen=set()
    for x in s.split(","):
        a,c=x.split(":")
        i,n=int(a),int(c)
        if n<=0 or i in seen:
            raise ValueError("nonpositive count or duplicate pair")
        seen.add(i)
        z.append((i,n))
    return tuple(z)

def check(condition, name, detail):
    if not condition:
        raise ValueError("%s:%s"%(name,detail))

def load_mapping(path):
    e2s={}
    seen=set()
    emeta={}
    with open(path,encoding="ascii") as f:
        for line in f:
            d=fields(line,"MAP")
            check(d["json"]=="0","map_json",line[:80])
            e,s=int(d["engine_key"]),int(d["stage_key"])
            check(e not in e2s and s not in seen,"bijection_collision","engine_key=%s stage_key=%s"%(e,s))
            e2s[e]=s
            seen.add(s)
            emeta[s]=(int(d["M"]),int(d["size"]),int(d["W"]))
    return e2s,emeta

def load_stage(path,m0):
    tails=[]
    rows=[]
    head=None
    with open(path,"r",encoding="ascii") as f:
        for line in f:
            if line.startswith("HEAD|"):
                check(head is None,"stage_head","duplicate")
                head=fields(line,"HEAD")
                check(int(head["m0"])==m0,"stage_cutoff",str(head))
            elif line.startswith("T|"):
                d=fields(line,"T")
                check(int(d["t"])==len(tails),"tail_id","out of order")
                tails.append((int(d["M"]),pairs(d["back"])))
            elif line.startswith("R|"):
                d=fields(line,"R")
                check(int(d["id"])==len(rows),"stage_row_id","out of order")
                rows.append({"key":int(d["key"]),"M":int(d["M"]),
                             "size":int(d["size"]),"W":int(d["W"]),
                             "flat":int(d["flat"]),"core":pairs(d["core"]),
                             "tail":pairs(d["tail"]),
                             "ncore":int(d["ncore"]),"ntail":int(d["ntail"])})
            else:
                raise ValueError("unknown Stage A record")
    check(head is not None and len(rows)==int(head["classes"]) and len(tails)==int(head["tailsets"]),
          "stage_structure","header/count mismatch")
    keys=[x["key"] for x in rows]
    check(len(set(keys))==len(keys),"stage_duplicate_class","duplicate Stage A key")
    for r in rows:
        check(sum(c for _,c in r["core"])==r["ncore"],"stage_ncore",str(r["key"]))
        check(sum(c for _,c in r["tail"])==r["ntail"],"stage_ntail",str(r["key"]))
        check(r["flat"]+r["ncore"]+r["ntail"]==72,"stage_degree",str(r["key"]))
    return rows,tails,keys

def main():
    stage_path,engine_path,map_path,m0s=sys.argv[1:]
    m0=int(m0s)
    e2s,emeta=load_mapping(map_path)
    rows,tails,ids=load_stage(stage_path,m0)
    bystage={r["key"]:r for r in rows}
    check(len(e2s)==len(rows),"bijection_cardinality",
          "%d engine mapped vs %d cleanroom"%(len(e2s),len(rows)))
    check(set(e2s.values())==set(bystage),"bijection_onto","sets of class keys differ")
    for s, m in emeta.items():
        r=bystage[s]
        check(m==(r["M"],r["size"],r["W"]),"mapped_metadata",
              "stage_key=%s expected=%s engine=%s"%(s,(r["M"],r["size"],r["W"]),m))
    print("STAGEB_BIJECTION|PASS|m0=%d|classes=%d|injective=1|surjective=1|M_size_W_match=1"%
          (m0,len(rows)),flush=True)

    cache={}
    def stagetail(tid):
        t=cache.get(tid)
        if t is not None:
            return t
        m, backs=tails[tid]
        t=(m,tuple(sorted((ids[b],c) for b,c in backs)))
        cache[tid]=t
        return t

    def compare_one(engine,engine_tail):
        ekey=int(engine["key"])
        skey=e2s.get(ekey)
        check(skey is not None,"engine_key_missing",str(ekey))
        want=bystage[skey]
        for attr in ("M","size","W","flat"):
            check(int(engine[attr])==want[attr],"row_"+attr,
                  "engine_key=%s stage_key=%s got=%s expected=%s"%
                  (ekey,skey,engine[attr],want[attr]))
        observed_core=Counter()
        for eid,c in pairs(engine["core"]):
            check(eid in e2s,"engine_core_bad_target","src=%d dst=%d"%(ekey,eid))
            observed_core[e2s[eid]]+=c
        expected_core=Counter({ids[b]:c for b,c in want["core"]})
        check(observed_core==expected_core,"row_core",
              "engine_key=%d stage_key=%d observed=%s wanted=%s"%
              (ekey,skey,str(observed_core.most_common(12)),str(expected_core.most_common(12))))
        observed_tail=Counter()
        for t in engine_tail:
            m=int(t["M"])
            back=Counter()
            for val in (x for x in t["back"].split(",") if x):
                eid=int(val)
                check(eid in e2s,"engine_tail_bad_target","src=%d dst=%d"%(ekey,eid))
                back[e2s[eid]]+=1
            observed_tail[(m,tuple(sorted(back.items())))]+=1
        expected_tail=Counter()
        for tid,c in want["tail"]:
            expected_tail[stagetail(tid)]+=c
        check(observed_tail==expected_tail,"row_tail",
              "engine_key=%d stage_key=%d observed_tail_entries=%d want=%d first_observed=%s first_expected=%s"%
              (ekey,skey,sum(observed_tail.values()),sum(expected_tail.values()),
               str(observed_tail.most_common(3)),str(expected_tail.most_common(3))))
        check(int(engine["flat"])+sum(observed_core.values())+len(engine_tail)==72,
              "engine_degree",str(ekey))
        return len(engine_tail),sum(len(t["back"].split(",")) for t in engine_tail if t["back"])

    count=0
    n_tail=0
    n_back=0
    observed_engine_keys=set()
    hdr=None
    current=None
    tail=[]
    end=None
    with open(engine_path,"r",encoding="ascii") as f:
        for line in f:
            if line.startswith("HEAD|"):
                check(hdr is None,"engine_head","duplicate")
                hdr=fields(line,"HEAD")
                check(int(hdr["m0"])==m0,"engine_m0",str(hdr))
            elif line.startswith("C|"):
                if current is not None:
                    a,b=compare_one(current,tail)
                    n_tail+=a
                    n_back+=b
                    count+=1
                current=fields(line,"C")
                key=int(current["key"])
                check(key not in observed_engine_keys,"engine_duplicate_key",str(key))
                observed_engine_keys.add(key)
                tail=[]
            elif line.startswith("T|"):
                check(current is not None,"tail_without_core",line[:60])
                tail.append(fields(line,"T"))
            elif line.startswith("END|"):
                check(end is None and current is not None,"end","bad end")
                a,b=compare_one(current,tail)
                n_tail+=a
                n_back+=b
                count+=1
                current=None
                end=fields(line,"END")
            else:
                raise ValueError("unknown engine record "+line[:80])
    check(hdr is not None and end is not None and current is None,"engine_end","missing end")
    check(count==len(rows)==int(hdr["classes"])==int(end["classes"]),
          "row_total","%d vs %d"%(count,len(rows)))
    check(len(observed_engine_keys)==len(e2s) and observed_engine_keys==set(e2s),
          "engine_keys_coverage","unmatched engine key")
    check(n_tail==int(end["tail"]) and n_back==int(end["back"]),
          "tail_total","observed %s,%s expected %s,%s"%(n_tail,n_back,end["tail"],end["back"]))
    print("STAGEB_FULL_ROWS|PASS|m0=%d|classes=%d|tail_entries=%d|back_moves=%d|all_class_rows_equal=1"%
          (m0,count,n_tail,n_back),flush=True)
    print("STAGEB_VERDICT|PASS|m0=%d|algorithmic_comparison=EXHAUSTIVE|stageA_author_separate=DISCLOSED|r9_executions=0"%m0,flush=True)

if __name__=="__main__":
    try:
        main()
    except (ValueError, KeyError, IndexError, OSError) as exc:
        print("STAGEB_VERDICT|FAIL|reason=%s"%str(exc),file=sys.stderr,flush=True)
        sys.exit(1)
