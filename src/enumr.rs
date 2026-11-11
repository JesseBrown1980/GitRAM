//! Enumeration of all gauge orbits with M <= m0 by depth-first search over the 13 free links of
//! the tree gauge, pruned by the partial magnetic number, one orbit per conjugation-minimal leaf.

use crate::lat::{Cfg, Ctx, NFREE, NL, NS};
use std::sync::atomic::{AtomicUsize, Ordering};

pub const MMAX: usize = 40;
const PREFIX: usize = 4;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Rep {
    pub m: u32,
    pub key: u64,
    pub size: u32,
    pub w: u32,
}

pub struct EnumOut {
    pub orbits: [u64; MMAX],
    pub weight: [u64; MMAX],
    pub by_w: [[u64; 7]; MMAX],
    pub classes: [u64; MMAX],
    pub class_orbits: [u64; MMAX],
    pub class_weight: [u64; MMAX],
    pub reps: Vec<Rep>,
    /// every nonflat orbit (key, M, w), only when requested
    pub all: Vec<(u64, u32, u32)>,
    pub leaves: u64,
}

impl EnumOut {
    fn new() -> EnumOut {
        EnumOut {
            orbits: [0; MMAX],
            weight: [0; MMAX],
            by_w: [[0; 7]; MMAX],
            classes: [0; MMAX],
            class_orbits: [0; MMAX],
            class_weight: [0; MMAX],
            reps: Vec::new(),
            all: Vec::new(),
            leaves: 0,
        }
    }
    fn merge(&mut self, o: EnumOut) {
        for m in 0..MMAX {
            self.orbits[m] += o.orbits[m];
            self.weight[m] += o.weight[m];
            self.classes[m] += o.classes[m];
            self.class_orbits[m] += o.class_orbits[m];
            self.class_weight[m] += o.class_weight[m];
            for k in 0..7 {
                self.by_w[m][k] += o.by_w[m][k];
            }
        }
        self.reps.extend(o.reps);
        self.all.extend(o.all);
        self.leaves += o.leaves;
    }
}

struct Plan {
    /// plaquettes completed when the free link of this depth is assigned
    complete: Vec<Vec<usize>>,
}

fn plan(ctx: &Ctx) -> Plan {
    let mut depth_of = [usize::MAX; NL];
    for (d, &(_, l, _)) in ctx.free.iter().enumerate() {
        depth_of[l] = d;
    }
    let mut complete = vec![Vec::new(); NFREE];
    for p in 0..NS {
        let mut dmax: Option<usize> = None;
        for &l in ctx.plq[p].iter() {
            if depth_of[l] != usize::MAX {
                dmax = Some(dmax.map_or(depth_of[l], |x| x.max(depth_of[l])));
            }
        }
        complete[dmax.expect("every plaquette has a free link")].push(p);
    }
    Plan { complete }
}

struct Walker<'a> {
    ctx: &'a Ctx,
    plan: &'a Plan,
    m0: u32,
    keep_all: bool,
    out: EnumOut,
}

impl Walker<'_> {
    fn added(&self, u: &Cfg, d: usize) -> u32 {
        let mut a = 0;
        for &p in self.plan.complete[d].iter() {
            a += self.ctx.mag[self.ctx.hol(u, p) as usize];
        }
        a
    }

    fn leaf(&mut self, u: &Cfg, own: u64, m: u32) {
        self.out.leaves += 1;
        let (key, stab) = self.ctx.orbit_key(u);
        if key != own {
            return;
        }
        let w = 6 / stab;
        let mi = m as usize;
        self.out.orbits[mi] += 1;
        self.out.weight[mi] += u64::from(w);
        self.out.by_w[mi][w as usize] += 1;
        if m == 0 {
            return;
        }
        if self.keep_all {
            self.out.all.push((key, m, w));
        }
        let (ckey, size, w2) = self.ctx.class_info(u);
        assert!(w2 == w);
        if ckey == key {
            self.out.classes[mi] += 1;
            self.out.class_orbits[mi] += u64::from(size);
            self.out.class_weight[mi] += u64::from(size) * u64::from(w);
            self.out.reps.push(Rep { m, key, size, w });
        }
    }

    fn rec(&mut self, u: &mut Cfg, d: usize, own: u64, part: u32) {
        if d == NFREE {
            self.leaf(u, own, part);
            return;
        }
        let l = self.ctx.free[d].1;
        for v in 0..6u8 {
            u[l] = v;
            let p2 = part + self.added(u, d);
            if p2 <= self.m0 {
                self.rec(u, d + 1, own * 6 + u64::from(v), p2);
            }
        }
        u[l] = 0;
    }

    fn task(&mut self, t: usize) {
        let mut u: Cfg = [0u8; NL];
        let mut part = 0u32;
        let mut own = 0u64;
        let mut tt = t;
        let mut digits = [0u8; PREFIX];
        for d in (0..PREFIX).rev() {
            digits[d] = (tt % 6) as u8;
            tt /= 6;
        }
        for (d, &v) in digits.iter().enumerate() {
            u[self.ctx.free[d].1] = v;
            part += self.added(&u, d);
            own = own * 6 + u64::from(v);
        }
        if part <= self.m0 {
            self.rec(&mut u, PREFIX, own, part);
        }
    }
}

pub fn enumerate(ctx: &Ctx, m0: u32, threads: usize, keep_all: bool) -> EnumOut {
    let pl = plan(ctx);
    let ntask = 6usize.pow(PREFIX as u32);
    let next = AtomicUsize::new(0);
    let mut total = EnumOut::new();
    std::thread::scope(|s| {
        let mut hs = Vec::new();
        for _ in 0..threads {
            hs.push(s.spawn(|| {
                let mut w = Walker { ctx, plan: &pl, m0, keep_all, out: EnumOut::new() };
                loop {
                    let t = next.fetch_add(1, Ordering::SeqCst);
                    if t >= ntask {
                        break;
                    }
                    w.task(t);
                }
                w.out
            }));
        }
        for h in hs {
            total.merge(h.join().expect("enumeration thread"));
        }
    });
    total.reps.sort_unstable();
    total.all.sort_unstable();
    total
}
