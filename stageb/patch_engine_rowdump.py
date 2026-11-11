#!/usr/bin/env python3
"""Copy-free source-only instrumentation of v1 engine for Stage B.

Only adds a new subcommand. It does not modify the original row construction,
the existing row hash, the finite mathematical definitions or the frozen
published source. The instrumented diagnostic binary is NOT the frozen binary.
"""
import hashlib
import pathlib
import sys

def main():
    p=pathlib.Path(sys.argv[1])
    original=p.read_bytes()
    source=original.decode("utf-8")
    anchor="fn main() {\n"
    dispatch='        Some("rowhash") => rowhash_only(&args),\n'
    assert source.count(anchor)==1
    assert source.count(dispatch)==1
    source=source.replace(anchor, STAGEB_FUNCS + "\n" + anchor, 1)
    source=source.replace(dispatch,dispatch+'        Some("stageb-dump") => stageb_dump(&args),\n',1)
    p.write_text(source,encoding="utf-8",newline="\n")
    print("STAGEB_PATCH|PASS|original_sha256=%s|patched_sha256=%s" %
          (hashlib.sha256(original).hexdigest(),hashlib.sha256(source.encode()).hexdigest()))

STAGEB_FUNCS=r"""
// Stage B instrument-only row export, not part of the published engine.
// Preserves the original construction and row_hash. No coupling parameter.
fn stageb_dump(args: &[String]) {
    use std::io::{BufWriter, Write as _};
    let tab = tables();
    let (b, m0) = build(args, &tab);
    let outpath = format!("{}/stageb_engine_m{}.hbp", arg(args, "--dir", "."), m0);
    let f = std::fs::File::create(&outpath).expect("row dump output");
    let mut out = BufWriter::with_capacity(1 << 20, f);
    writeln!(out,"HEAD|m0={}|classes={}",m0,b.rows.len()).unwrap();
    let mut nt = 0usize;
    let mut nb = 0usize;
    for row in b.rows.iter() {
        let core = row.core.iter().map(|&(j,c)| format!("{}:{}",b.rows[j as usize].key,c))
                            .collect::<Vec<_>>().join(",");
        writeln!(out,"C|key={}|M={}|size={}|W={}|flat={}|core={}",
                 row.key,row.mag,row.norb,row.wt,row.nflat,core).unwrap();
        for t in row.tails.iter() {
            let back = t.back.iter().map(|&j| b.rows[j as usize].key.to_string())
                          .collect::<Vec<_>>().join(",");
            writeln!(out,"T|M={}|back={}",t.m,back).unwrap();
            nt += 1;
            nb += t.back.len();
        }
    }
    writeln!(out,"END|classes={}|tail={}|back={}",b.rows.len(),nt,nb).unwrap();
    out.flush().unwrap();
    println!("STAGEB_DUMP|file={}|classes={}|tail={}|back={}|rowhash={}|keyshash={}",
            outpath,b.rows.len(),nt,nb,b.rowhash,b.keyshash);
    // Independently recompute the label-free colour fingerprint from the
    // engine's in-memory rows, not the Stage A rows or fingerprints.
    let mut colour: Vec<String> = b.rows.iter().map(|r|format!("{},{},{},{}",
                      r.mag,r.norb,r.wt,r.nflat)).collect();
    for _ in 0..3 {
        let mut next = Vec::with_capacity(colour.len());
        for (i,r) in b.rows.iter().enumerate() {
            let mut core: Vec<String> = r.core.iter()
                .map(|&(j,c)|format!("{}:{}",colour[j as usize],c)).collect();
            core.sort_unstable();
            let mut tail: Vec<String> = Vec::with_capacity(r.tails.len());
            for t in r.tails.iter() {
                let mut backs: Vec<&str> = t.back.iter()
                    .map(|&j|colour[j as usize].as_str()).collect();
                backs.sort_unstable();
                tail.push(format!("{}:{}",t.m,backs.join(".")));
            }
            tail.sort_unstable();
            next.push(sha256_hex(format!("{}|{}|{}",colour[i],
                          core.join(","),tail.join(",")).as_bytes()));
        }
        colour=next;
    }
    colour.sort_unstable();
    let fp=sha256_hex(colour.join("\n").as_bytes());
    colour.dedup();
    println!("STAGEB_FINGERPRINT|m0={}|sha256={}|distinct={}",m0,fp,colour.len());
}
"""

if __name__=="__main__":
    main()
