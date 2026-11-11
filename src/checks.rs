//! Named checks. Each returns (pass, detail); nothing here panics on a failed check.

use crate::enumr::{EnumOut, MMAX};
use crate::lat::{Cfg, Ctx, NS};
use crate::rows::{direct_row, lookup, Table, MISS};
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Rng(pub u64);
impl Rng {
    pub fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    pub fn below(&mut self, n: u64) -> u64 {
        (self.next() >> 11) % n
    }
    pub fn gauge(&mut self) -> [u8; NS] {
        let mut g = [0u8; NS];
        for x in g.iter_mut() {
            *x = self.below(6) as u8;
        }
        g
    }
}

pub struct Pred {
    pub orbits: [u64; MMAX],
    pub weight: [u64; MMAX],
    pub by_w: [[u64; 7]; MMAX],
}

/// Reads PREDICT-STAGE-A.hbp (rows PREDICT|M=..|orbits=..|weight=..|orbits_w1=..|...).
pub fn read_pred(path: &str) -> Pred {
    let text = std::fs::read_to_string(path).expect("prediction file");
    let mut p = Pred { orbits: [0; MMAX], weight: [0; MMAX], by_w: [[0; 7]; MMAX] };
    for line in text.lines() {
        let f: Vec<&str> = line.split('|').collect();
        if f[0] != "PREDICT" {
            continue;
        }
        let mut kv: HashMap<&str, u64> = HashMap::new();
        for x in &f[1..] {
            if let Some((k, v)) = x.split_once('=') {
                kv.insert(k, v.parse().expect("integer field"));
            }
        }
        let m = kv["M"] as usize;
        p.orbits[m] = kv["orbits"];
        p.weight[m] = kv["weight"];
        p.by_w[m][1] = kv["orbits_w1"];
        p.by_w[m][2] = kv["orbits_w2"];
        p.by_w[m][3] = kv["orbits_w3"];
        p.by_w[m][6] = kv["orbits_w6"];
    }
    p
}

/// C1: enumerated orbits, weight and weight-type split equal the independent count for every M <= m0.
pub fn c1_prediction(p: &Pred, e: &EnumOut, m0: u32) -> (bool, String) {
    let mut bad = Vec::new();
    for m in 0..=(m0 as usize) {
        if e.orbits[m] != p.orbits[m] || e.weight[m] != p.weight[m] || e.by_w[m] != p.by_w[m] {
            bad.push(format!("M{}:enum={}/{}:pred={}/{}", m, e.orbits[m], e.weight[m], p.orbits[m], p.weight[m]));
        }
    }
    (bad.is_empty(), if bad.is_empty() { format!("all_M_0_to_{}_equal", m0) } else { bad.join(",") })
}

/// C2: class sizes and class weights of the stored table sum to the independent count for every M in 1..=m0.
pub fn c2_class_sums(p: &Pred, t: &Table) -> (bool, String) {
    let mut o = [0u64; MMAX];
    let mut w = [0u64; MMAX];
    for r in t.reps.iter() {
        o[r.m as usize] += u64::from(r.size);
        w[r.m as usize] += u64::from(r.size) * u64::from(r.w);
    }
    let mut bad = Vec::new();
    for m in 1..=(t.m0 as usize) {
        if o[m] != p.orbits[m] || w[m] != p.weight[m] {
            bad.push(format!("M{}:table={}/{}:pred={}/{}", m, o[m], w[m], p.orbits[m], p.weight[m]));
        }
    }
    (bad.is_empty(), if bad.is_empty() { "class_sums_equal".to_string() } else { bad.join(",") })
}

/// C4: the row does not depend on the representative.
/// all = true: every class, all 48 symmetry images each under a random gauge transformation, plus two
/// pure gauge images.  all = false: classes with id % step == offset, three variants each (another orbit of
/// the class when the class has more than one, another configuration of the orbit, both).
pub fn c4_rep(ctx: &Ctx, t: &Table, all: bool, step: usize, offset: usize, threads: usize) -> (bool, String) {
    let n = t.rows.len();
    let next = AtomicUsize::new(0);
    let mut fails = 0u64;
    let mut done = 0u64;
    let mut classes = 0u64;
    let mut first = String::new();
    std::thread::scope(|s| {
        let mut hs = Vec::new();
        for _ in 0..threads {
            hs.push(s.spawn(|| {
                let (mut f, mut d, mut c, mut fst) = (0u64, 0u64, 0u64, String::new());
                loop {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= n {
                        break;
                    }
                    if !all && i % step != offset {
                        continue;
                    }
                    c += 1;
                    let mut rng = Rng(0x9E37_79B9_7F4A_7C15 ^ ((i as u64 + 1) * 0x1000_0000_01B3));
                    let rep = ctx.cfg_from_key(t.reps[i].key);
                    let want = t.expand(i);
                    let mut variants: Vec<Cfg> = Vec::new();
                    if all {
                        for sy in 0..48 {
                            variants.push(ctx.apply_gauge(&ctx.apply_sym(sy, &rep), &rng.gauge()));
                        }
                        variants.push(ctx.apply_gauge(&rep, &rng.gauge()));
                        variants.push(ctx.apply_gauge(&rep, &rng.gauge()));
                    } else {
                        let own = ctx.orbit_key(&rep).0;
                        let mut other = ctx.apply_sym(rng.below(48) as usize, &rep);
                        for sy in 0..48 {
                            let c2 = ctx.apply_sym(sy, &rep);
                            if ctx.orbit_key(&c2).0 != own {
                                other = c2;
                                if rng.below(4) == 0 {
                                    break;
                                }
                            }
                        }
                        variants.push(other);
                        variants.push(ctx.apply_gauge(&rep, &rng.gauge()));
                        variants.push(ctx.apply_gauge(&ctx.apply_sym(rng.below(48) as usize, &rep), &rng.gauge()));
                    }
                    for v in variants.iter() {
                        d += 1;
                        let (got, _) = direct_row(ctx, &t.index, t.m0, v);
                        let lab = lookup(&t.index, ctx.class_key(v));
                        if got != want || lab != i as u32 {
                            f += 1;
                            if fst.is_empty() {
                                fst = format!("class{}", i);
                            }
                        }
                    }
                }
                (f, d, c, fst)
            }));
        }
        for h in hs {
            let (f, d, c, fst) = h.join().expect("c4 thread");
            fails += f;
            done += d;
            classes += c;
            if first.is_empty() {
                first = fst;
            }
        }
    });
    (fails == 0, format!("classes={}:variants={}:mismatches={}:first={}", classes, done, fails, if first.is_empty() { "none" } else { &first }))
}

/// C5: orbit-level detailed balance w_i c_ij = w_j c_ji on every pair of nonflat orbits with M <= m0.
pub fn c5_orbit_balance(ctx: &Ctx, all: &[(u64, u32, u32)], m0: u32, threads: usize) -> (bool, String) {
    let n = all.len();
    let next = AtomicUsize::new(0);
    let mut edges: Vec<(u32, u32, u32)> = Vec::new();
    let mut missing = 0u64;
    std::thread::scope(|s| {
        let mut hs = Vec::new();
        for _ in 0..threads {
            hs.push(s.spawn(|| {
                let mut out: Vec<(u32, u32, u32)> = Vec::new();
                let mut miss = 0u64;
                loop {
                    let i = next.fetch_add(1, Ordering::SeqCst);
                    if i >= n {
                        break;
                    }
                    let u = ctx.cfg_from_key(all[i].0);
                    let mut js: Vec<u32> = Vec::new();
                    for mv in 0..72 {
                        let v = ctx.apply_move(&u, mv);
                        let m = ctx.mag_total(&v);
                        if m > 0 && m <= m0 {
                            let k = ctx.orbit_key(&v).0;
                            match all.binary_search_by(|e| e.0.cmp(&k)) {
                                Ok(j) => js.push(j as u32),
                                Err(_) => miss += 1,
                            }
                        }
                    }
                    js.sort_unstable();
                    let mut a = 0;
                    while a < js.len() {
                        let mut b = a;
                        while b < js.len() && js[b] == js[a] {
                            b += 1;
                        }
                        out.push((i as u32, js[a], (b - a) as u32));
                        a = b;
                    }
                }
                (out, miss)
            }));
        }
        for h in hs {
            let (o, ms) = h.join().expect("c5 thread");
            edges.extend(o);
            missing += ms;
        }
    });
    edges.sort_unstable();
    let mut bad = 0u64;
    for &(i, j, c) in edges.iter() {
        let lo = edges.partition_point(|e| (e.0, e.1) < (j, i));
        if lo < edges.len() && edges[lo].0 == j && edges[lo].1 == i {
            let c2 = edges[lo].2;
            if u64::from(all[i as usize].2) * u64::from(c) != u64::from(all[j as usize].2) * u64::from(c2) {
                bad += 1;
            }
        } else {
            bad += 1;
        }
    }
    (bad == 0 && missing == 0, format!("orbits={}:pairs={}:violations={}:unknown_targets={}", n, edges.len(), bad, missing))
}

/// C6: class-level detailed balance W'_A c_(A->B) = W'_B c_(B->A) on every core pair.
pub fn c6_class_balance(t: &Table) -> (bool, String) {
    let mut bad = 0u64;
    let mut pairs = 0u64;
    let wt = |i: usize| u64::from(t.reps[i].size) * u64::from(t.reps[i].w);
    for (a, r) in t.rows.iter().enumerate() {
        for &(b, c) in r.core.iter() {
            pairs += 1;
            if b == MISS {
                bad += 1;
                continue;
            }
            let rb = &t.rows[b as usize].core;
            match rb.binary_search_by(|e| e.0.cmp(&(a as u32))) {
                Ok(k) => {
                    if wt(a) * u64::from(c) != wt(b as usize) * u64::from(rb[k].1) {
                        bad += 1;
                    }
                }
                Err(_) => bad += 1,
            }
        }
    }
    (bad == 0, format!("pairs={}:violations={}", pairs, bad))
}

/// C7: two-step balance through the tail.  T_AB(m) = number of (move A -> tail orbit k with M_k = m,
/// move k -> B) pairs; W'_A T_AB(m) = W'_B T_BA(m) for all core A, B and every m.
pub fn c7_two_step(t: &Table, threads: usize) -> (bool, String) {
    let n = t.rows.len();
    let grouped: Vec<(u32, Vec<(u32, u32)>)> = t
        .tails
        .iter()
        .map(|c| {
            let mut g: Vec<(u32, u32)> = Vec::new();
            for &b in &c[1..] {
                match g.last_mut() {
                    Some(l) if l.0 == b => l.1 += 1,
                    _ => g.push((b, 1)),
                }
            }
            (c[0], g)
        })
        .collect();
    let nb = 1 + n / 1500;
    let next = AtomicUsize::new(0);
    let mut bad = 0u64;
    let mut keys = 0u64;
    let wt = |i: usize| i64::from(t.reps[i].size) * i64::from(t.reps[i].w);
    std::thread::scope(|s| {
        let mut hs = Vec::new();
        for _ in 0..threads {
            hs.push(s.spawn(|| {
                let (mut b, mut k) = (0u64, 0u64);
                loop {
                    let blk = next.fetch_add(1, Ordering::SeqCst);
                    if blk >= nb {
                        break;
                    }
                    let mut map: HashMap<(u32, u32, u32), i64> = HashMap::new();
                    for (a, r) in t.rows.iter().enumerate() {
                        let a32 = a as u32;
                        for &(tid, mult) in r.tail.iter() {
                            let (mk, g) = &grouped[tid as usize];
                            for &(bb, cnt) in g.iter() {
                                if bb == a32 || bb == MISS {
                                    continue;
                                }
                                let (lo, hi) = if a32 < bb { (a32, bb) } else { (bb, a32) };
                                if lo as usize % nb != blk {
                                    continue;
                                }
                                let v = wt(a) * i64::from(mult) * i64::from(cnt);
                                *map.entry((lo, hi, *mk)).or_insert(0) += if a32 < bb { v } else { -v };
                            }
                        }
                    }
                    k += map.len() as u64;
                    b += map.values().filter(|v| **v != 0).count() as u64;
                }
                (b, k)
            }));
        }
        for h in hs {
            let (b, k) = h.join().expect("c7 thread");
            bad += b;
            keys += k;
        }
    });
    (bad == 0, format!("triples={}:violations={}", keys, bad))
}

/// C8: every row accounts for 72 moves; every tail set has M_k > m0; every referenced class exists.
pub fn c8_structure(t: &Table) -> (bool, String) {
    let mut bad = 0u64;
    for r in t.rows.iter() {
        let s: u32 = r.flat + r.core.iter().map(|e| e.1).sum::<u32>() + r.tail.iter().map(|e| e.1).sum::<u32>();
        if s != 72 {
            bad += 1;
        }
        if r.core.iter().any(|e| e.0 == MISS) {
            bad += 1;
        }
    }
    for c in t.tails.iter() {
        if c[0] <= t.m0 || c[1..].iter().any(|b| *b == MISS) {
            bad += 1;
        }
    }
    (bad == 0, format!("rows={}:tailsets={}:violations={}", t.rows.len(), t.tails.len(), bad))
}
