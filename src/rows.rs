//! Row reconstruction, canonical serialization and label-free fingerprint.

use crate::enumr::Rep;
use crate::lat::{Cfg, Ctx};
use crate::sha::{sha256_hex, Sha256};
use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub const MISS: u32 = u32::MAX;

/// Row computed directly from one configuration, nothing shared or cached.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DRow {
    pub m: u32,
    pub size: u32,
    pub weight: u32,
    pub flat: u32,
    /// (core class id, number of moves), sorted by id
    pub core: Vec<(u32, u32)>,
    /// one entry per move landing in the tail: [M_k, back ids sorted with multiplicity]; sorted
    pub tail: Vec<Vec<u32>>,
}

/// Stored row: tail entries refer to interned back sets.
pub struct Row {
    pub flat: u32,
    pub core: Vec<(u32, u32)>,
    /// (tail set id, number of moves), sorted by id
    pub tail: Vec<(u32, u32)>,
}

pub struct Table {
    pub m0: u32,
    pub reps: Vec<Rep>,
    /// (class key, id) sorted by key
    pub index: Vec<(u64, u32)>,
    pub rows: Vec<Row>,
    /// tail sets: [M_k, back ids sorted with multiplicity], sorted lexicographically
    pub tails: Vec<Vec<u32>>,
    pub misses: u64,
}

pub fn build_index(reps: &[Rep]) -> Vec<(u64, u32)> {
    let mut ix: Vec<(u64, u32)> = reps.iter().enumerate().map(|(i, r)| (r.key, i as u32)).collect();
    ix.sort_unstable();
    ix
}

pub fn lookup(index: &[(u64, u32)], key: u64) -> u32 {
    match index.binary_search_by(|e| e.0.cmp(&key)) {
        Ok(i) => index[i].1,
        Err(_) => MISS,
    }
}

fn group(mut ids: Vec<u32>) -> Vec<(u32, u32)> {
    ids.sort_unstable();
    let mut out: Vec<(u32, u32)> = Vec::new();
    for id in ids {
        match out.last_mut() {
            Some(l) if l.0 == id => l.1 += 1,
            _ => out.push((id, 1)),
        }
    }
    out
}

/// The row of the class of configuration u at cutoff m0, straight from the definition.
/// Returns the row and the number of core look-ups that found no enumerated class.
pub fn direct_row(ctx: &Ctx, index: &[(u64, u32)], m0: u32, u: &Cfg) -> (DRow, u64) {
    let (_, size, w) = ctx.class_info(u);
    let m = ctx.mag_total(u);
    let mut flat = 0u32;
    let mut core_ids = Vec::new();
    let mut tail = Vec::new();
    let mut misses = 0u64;
    for mv in 0..72 {
        let v = ctx.apply_move(u, mv);
        let mk = ctx.mag_total(&v);
        if mk == 0 {
            flat += 1;
        } else if mk <= m0 {
            let id = lookup(index, ctx.class_key(&v));
            if id == MISS {
                misses += 1;
            }
            core_ids.push(id);
        } else {
            let mut back = Vec::new();
            for mv2 in 0..72 {
                let v2 = ctx.apply_move(&v, mv2);
                let m2 = ctx.mag_total(&v2);
                if m2 > 0 && m2 <= m0 {
                    let id = lookup(index, ctx.class_key(&v2));
                    if id == MISS {
                        misses += 1;
                    }
                    back.push(id);
                }
            }
            back.sort_unstable();
            let mut e = Vec::with_capacity(back.len() + 1);
            e.push(mk);
            e.extend(back);
            tail.push(e);
        }
    }
    tail.sort_unstable();
    (DRow { m, size, weight: size * w, flat, core: group(core_ids), tail }, misses)
}

pub fn build(ctx: &Ctx, m0: u32, reps: Vec<Rep>, threads: usize) -> Table {
    let index = build_index(&reps);
    let n = reps.len();
    let intern: Mutex<HashMap<Vec<u32>, u32>> = Mutex::new(HashMap::new());
    let next = AtomicUsize::new(0);
    let mut parts: Vec<(usize, Row)> = Vec::with_capacity(n);
    let mut misses = 0u64;
    std::thread::scope(|s| {
        let mut hs = Vec::new();
        for _ in 0..threads {
            hs.push(s.spawn(|| {
                let mut out: Vec<(usize, Row)> = Vec::new();
                let mut miss = 0u64;
                loop {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= n {
                        break;
                    }
                    let u = ctx.cfg_from_key(reps[i].key);
                    let (d, ms) = direct_row(ctx, &index, m0, &u);
                    miss += ms;
                    assert!(d.m == reps[i].m && d.size == reps[i].size && d.weight == reps[i].size * reps[i].w);
                    let mut tids = Vec::with_capacity(d.tail.len());
                    {
                        let mut g = intern.lock().expect("intern lock");
                        for e in d.tail {
                            let nid = g.len() as u32;
                            tids.push(*g.entry(e).or_insert(nid));
                        }
                    }
                    out.push((i, Row { flat: d.flat, core: d.core, tail: group(tids) }));
                }
                (out, miss)
            }));
        }
        for h in hs {
            let (o, ms) = h.join().expect("row thread");
            parts.extend(o);
            misses += ms;
        }
    });
    parts.sort_unstable_by_key(|p| p.0);
    let mut rows: Vec<Row> = parts.into_iter().map(|p| p.1).collect();
    // canonical numbering of the tail sets: lexicographic order of [M_k, ids...]
    let map = intern.into_inner().expect("intern");
    let mut sets: Vec<(Vec<u32>, u32)> = map.into_iter().collect();
    sets.sort_unstable();
    let mut remap = vec![0u32; sets.len()];
    let mut tails = Vec::with_capacity(sets.len());
    for (newid, (content, old)) in sets.into_iter().enumerate() {
        remap[old as usize] = newid as u32;
        tails.push(content);
    }
    for r in rows.iter_mut() {
        for e in r.tail.iter_mut() {
            e.0 = remap[e.0 as usize];
        }
        r.tail.sort_unstable();
    }
    Table { m0, reps, index, rows, tails, misses }
}

impl Table {
    /// Stored row expanded to the direct form.
    pub fn expand(&self, id: usize) -> DRow {
        let r = &self.rows[id];
        let mut tail = Vec::new();
        for &(t, mult) in r.tail.iter() {
            for _ in 0..mult {
                tail.push(self.tails[t as usize].clone());
            }
        }
        tail.sort_unstable();
        let rep = &self.reps[id];
        DRow { m: rep.m, size: rep.size, weight: rep.size * rep.w, flat: r.flat, core: r.core.clone(), tail }
    }

    /// (core classes, core-to-core nonzero entries, sum of flat counts, tail entries, total back size)
    pub fn totals(&self) -> (u64, u64, u64, u64, u64) {
        let mut nz = 0u64;
        let mut flat = 0u64;
        let mut te = 0u64;
        let mut back = 0u64;
        for r in self.rows.iter() {
            nz += r.core.len() as u64;
            flat += u64::from(r.flat);
            for &(t, mult) in r.tail.iter() {
                te += u64::from(mult);
                back += u64::from(mult) * (self.tails[t as usize].len() as u64 - 1);
            }
        }
        (self.rows.len() as u64, nz, flat, te, back)
    }

    /// Canonical serialization; returns its SHA-256.
    pub fn write(&self, path: &str) -> std::io::Result<String> {
        let f = std::fs::File::create(path)?;
        let mut w = std::io::BufWriter::with_capacity(1 << 20, f);
        let mut h = Sha256::new();
        let mut emit = |line: &str, w: &mut std::io::BufWriter<std::fs::File>| -> std::io::Result<()> {
            h.update(line.as_bytes());
            w.write_all(line.as_bytes())
        };
        emit(&format!("HEAD|stage=A|m0={}|classes={}|tailsets={}|json=0\n", self.m0, self.rows.len(), self.tails.len()), &mut w)?;
        let pairs = |v: &[(u32, u32)]| -> String {
            if v.is_empty() {
                return "-".to_string();
            }
            let mut s = String::new();
            for (i, (a, c)) in v.iter().enumerate() {
                if i > 0 {
                    s.push(',');
                }
                s.push_str(&format!("{}:{}", a, c));
            }
            s
        };
        for (t, c) in self.tails.iter().enumerate() {
            let g = group(c[1..].to_vec());
            emit(&format!("T|t={}|M={}|n={}|back={}|json=0\n", t, c[0], c.len() - 1, pairs(&g)), &mut w)?;
        }
        for (id, r) in self.rows.iter().enumerate() {
            let rep = &self.reps[id];
            let nt: u32 = r.tail.iter().map(|e| e.1).sum();
            let nc: u32 = r.core.iter().map(|e| e.1).sum();
            emit(
                &format!(
                    "R|id={}|key={}|M={}|size={}|w={}|W={}|flat={}|ncore={}|ntail={}|core={}|tail={}|json=0\n",
                    id,
                    rep.key,
                    rep.m,
                    rep.size,
                    rep.w,
                    rep.size * rep.w,
                    r.flat,
                    nc,
                    nt,
                    pairs(&r.core),
                    pairs(&r.tail)
                ),
                &mut w,
            )?;
        }
        w.flush()?;
        Ok(h.hex())
    }

    /// Label-free fingerprint: (fingerprint, number of distinct colour_3 values).
    pub fn fingerprint(&self, threads: usize) -> (String, usize) {
        let n = self.rows.len();
        let mut col: Vec<String> = (0..n)
            .map(|i| format!("{},{},{},{}", self.reps[i].m, self.reps[i].size, self.reps[i].size * self.reps[i].w, self.rows[i].flat))
            .collect();
        for _round in 0..3 {
            let next = AtomicUsize::new(0);
            let colr = &col;
            let mut parts: Vec<(usize, String)> = Vec::with_capacity(n);
            std::thread::scope(|s| {
                let mut hs = Vec::new();
                for _ in 0..threads {
                    hs.push(s.spawn(|| {
                        let mut out = Vec::new();
                        loop {
                            let i = next.fetch_add(1, Ordering::SeqCst);
                            if i >= n {
                                break;
                            }
                            out.push((i, self.colour_step(i, colr)));
                        }
                        out
                    }));
                }
                for h in hs {
                    parts.extend(h.join().expect("colour thread"));
                }
            });
            parts.sort_unstable_by_key(|p| p.0);
            col = parts.into_iter().map(|p| p.1).collect();
        }
        col.sort_unstable();
        let joined = col.join("\n");
        let fp = sha256_hex(joined.as_bytes());
        col.dedup();
        (fp, col.len())
    }

    fn colour_step(&self, i: usize, col: &[String]) -> String {
        let r = &self.rows[i];
        let mut core: Vec<String> = r.core.iter().map(|&(b, c)| format!("{}:{}", col[b as usize], c)).collect();
        core.sort_unstable();
        let mut tail: Vec<String> = Vec::new();
        for &(t, mult) in r.tail.iter() {
            let c = &self.tails[t as usize];
            let mut back: Vec<&str> = c[1..].iter().map(|&b| col[b as usize].as_str()).collect();
            back.sort_unstable();
            let s = format!("{}:{}", c[0], back.join("."));
            for _ in 0..mult {
                tail.push(s.clone());
            }
        }
        tail.sort_unstable();
        let text = format!("{}|{}|{}", col[i], core.join(","), tail.join(","));
        sha256_hex(text.as_bytes())
    }
}
