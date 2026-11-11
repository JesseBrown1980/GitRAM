//! Lattice, group and symmetry primitives. Exact integers only.
//!
//! Element encoding (permutations of {0,1,2} as images of 0,1,2):
//!   0 = e = [0,1,2]; 1 = (0 1) = [1,0,2]; 2 = (0 2) = [2,1,0]; 3 = (1 2) = [0,2,1];
//!   4 = [1,2,0]; 5 = [2,0,1].  Product (a*b)(i) = a(b(i)).
//! Site index s = x + 3*y (x in 0..3, y in 0..4).  Link index l = 2*s + d, d = 0 for the
//! link (x,y)->(x+1,y), d = 1 for the link (x,y)->(x,y+1).

pub const NL: usize = 24;
pub const NS: usize = 12;
pub const NFREE: usize = 13;
pub type Cfg = [u8; NL];

const PERMS: [[u8; 3]; 6] = [[0, 1, 2], [1, 0, 2], [2, 1, 0], [0, 2, 1], [1, 2, 0], [2, 0, 1]];

/// Source-level mutations used by the controls (0 = none).
pub const MUT_MUL_LOCAL: u8 = 1; // product order swapped inside the gauge fixing only
pub const MUT_NO_INVERSION: u8 = 2; // reversed links not inverted by reflections
pub const MUT_PLAQ_LINK: u8 = 3; // plaquette 0: third link taken forward instead of backward
pub const MUT_PLAQ_LINK_TOP: u8 = 9; // plaquette 9 = (0,3): third link taken forward instead of backward
pub const MUT_MUL_GLOBAL: u8 = 7; // product order swapped everywhere (anti-isomorphism)
pub const MUT_PLAQ_REVERSE: u8 = 8; // plaquette 0 traversed the other way round

pub fn site(x: usize, y: usize) -> usize {
    (x % 3) + 3 * (y % 4)
}
pub fn lx(x: usize, y: usize) -> usize {
    2 * site(x, y)
}
pub fn ly(x: usize, y: usize) -> usize {
    2 * site(x, y) + 1
}

pub struct Ctx {
    pub mul: [[u8; 6]; 6],
    pub inv: [u8; 6],
    pub mag: [u32; 6],
    /// conj[g][a] = g a g^-1
    pub conj: [[u8; 6]; 6],
    /// sym[s][target link] = (source link, invert)
    pub sym: Vec<[(u8, bool); NL]>,
    /// plaquette p (= site index of lower-left corner): the four links in traversal order
    pub plq: [[usize; 4]; NS],
    /// tree edges (from site, link, to site), in propagation order
    pub tree: [(usize, usize, usize); NS - 1],
    /// free links (from site, link, to site), most significant key digit first
    pub free: [(usize, usize, usize); NFREE],
    pub pow6: [u64; NFREE],
    pub mutation: u8,
}

impl Ctx {
    pub fn new(mutation: u8) -> Ctx {
        let mut mul = [[0u8; 6]; 6];
        for (a, row) in mul.iter_mut().enumerate() {
            for (b, cell) in row.iter_mut().enumerate() {
                let (p, q) = if mutation == MUT_MUL_GLOBAL { (b, a) } else { (a, b) };
                let c = [PERMS[p][PERMS[q][0] as usize], PERMS[p][PERMS[q][1] as usize], PERMS[p][PERMS[q][2] as usize]];
                let mut found = 6usize;
                for (k, pk) in PERMS.iter().enumerate() {
                    if *pk == c {
                        found = k;
                    }
                }
                assert!(found < 6);
                *cell = found as u8;
            }
        }
        let mut inv = [0u8; 6];
        for (a, row) in mul.iter().enumerate() {
            for (b, &c) in row.iter().enumerate() {
                if c == 0 {
                    inv[a] = b as u8;
                }
            }
        }
        let mag = [0u32, 2, 2, 2, 3, 3];
        let mut conj = [[0u8; 6]; 6];
        for (g, row) in conj.iter_mut().enumerate() {
            for (a, cell) in row.iter_mut().enumerate() {
                *cell = mul[mul[g][a] as usize][inv[g] as usize];
            }
        }
        let mut plq = [[0usize; 4]; NS];
        for y in 0..4 {
            for x in 0..3 {
                plq[site(x, y)] = [lx(x, y), ly(x + 1, y), lx(x, y + 1), ly(x, y)];
            }
        }
        // 48 symmetries: (x,y) -> (sx*x + tx, sy*y + ty)
        let mut sym = Vec::new();
        for rx in 0..2usize {
            for ry in 0..2usize {
                for tx in 0..3usize {
                    for ty in 0..4usize {
                        let phi = |x: usize, y: usize| -> (usize, usize) {
                            let nx = if rx == 1 { (3 - x % 3) % 3 } else { x % 3 };
                            let ny = if ry == 1 { (4 - y % 4) % 4 } else { y % 4 };
                            ((nx + tx) % 3, (ny + ty) % 4)
                        };
                        let noinv = mutation == MUT_NO_INVERSION;
                        let mut m = [(0u8, false); NL];
                        let mut seen = [false; NL];
                        for y in 0..4 {
                            for x in 0..3 {
                                let (px, py) = phi(x, y);
                                let (tl, iv) = if rx == 1 { (lx(px + 2, py), true) } else { (lx(px, py), false) };
                                m[tl] = (lx(x, y) as u8, iv && !noinv);
                                assert!(!seen[tl]);
                                seen[tl] = true;
                                let (tl, iv) = if ry == 1 { (ly(px, py + 3), true) } else { (ly(px, py), false) };
                                m[tl] = (ly(x, y) as u8, iv && !noinv);
                                assert!(!seen[tl]);
                                seen[tl] = true;
                            }
                        }
                        sym.push(m);
                    }
                }
            }
        }
        assert!(sym.len() == 48);
        // spanning tree: x links (0,0)->(1,0)->(2,0); y links (x,y)->(x,y+1) for y = 0,1,2
        let mut tree = [(0usize, 0usize, 0usize); NS - 1];
        tree[0] = (site(0, 0), lx(0, 0), site(1, 0));
        tree[1] = (site(1, 0), lx(1, 0), site(2, 0));
        let mut k = 2;
        for y in 0..3 {
            for x in 0..3 {
                tree[k] = (site(x, y), ly(x, y), site(x, y + 1));
                k += 1;
            }
        }
        // free links: x(2,0); x(0..3,1); x(0..3,2); x(0..3,3); y(0..3,3)
        let mut free = [(0usize, 0usize, 0usize); NFREE];
        free[0] = (site(2, 0), lx(2, 0), site(0, 0));
        let mut k = 1;
        for y in 1..4 {
            for x in 0..3 {
                free[k] = (site(x, y), lx(x, y), site(x + 1, y));
                k += 1;
            }
        }
        for x in 0..3 {
            free[k] = (site(x, 3), ly(x, 3), site(x, 0));
            k += 1;
        }
        let mut pow6 = [1u64; NFREE];
        for i in (0..NFREE - 1).rev() {
            pow6[i] = pow6[i + 1] * 6;
        }
        Ctx { mul, inv, mag, conj, sym, plq, tree, free, pow6, mutation }
    }

    #[inline]
    pub fn m(&self, a: u8, b: u8) -> u8 {
        self.mul[a as usize][b as usize]
    }

    /// Holonomy of plaquette p: U1 * U2 * U3^-1 * U4^-1.
    #[inline]
    pub fn hol(&self, u: &Cfg, p: usize) -> u8 {
        let q = &self.plq[p];
        let (a, b, c, d) = (u[q[0]], u[q[1]], u[q[2]], u[q[3]]);
        if (p == 0 && self.mutation == MUT_PLAQ_LINK) || (p == 9 && self.mutation == MUT_PLAQ_LINK_TOP) {
            return self.m(self.m(self.m(a, b), c), self.inv[d as usize]);
        }
        if p == 0 && self.mutation == MUT_PLAQ_REVERSE {
            return self.m(self.m(self.m(d, c), self.inv[b as usize]), self.inv[a as usize]);
        }
        self.m(self.m(self.m(a, b), self.inv[c as usize]), self.inv[d as usize])
    }

    pub fn mag_total(&self, u: &Cfg) -> u32 {
        let mut t = 0;
        for p in 0..NS {
            t += self.mag[self.hol(u, p) as usize];
        }
        t
    }

    /// Tree gauge: returns the 13 free-link values after fixing every tree link to e.
    pub fn gauge_fix(&self, u: &Cfg) -> [u8; NFREE] {
        let mut g = [0u8; NS];
        for &(a, l, b) in self.tree.iter() {
            g[b] = if self.mutation == MUT_MUL_LOCAL { self.m(u[l], g[a]) } else { self.m(g[a], u[l]) };
        }
        let mut v = [0u8; NFREE];
        for (i, &(a, l, b)) in self.free.iter().enumerate() {
            v[i] = self.m(self.m(g[a], u[l]), self.inv[g[b] as usize]);
        }
        v
    }

    /// Canonical key of the gauge orbit of u (minimum over the six global conjugations of the
    /// tree-gauge free values read as a base-6 number) and the stabiliser order.
    pub fn orbit_key(&self, u: &Cfg) -> (u64, u32) {
        let v = self.gauge_fix(u);
        let mut best = u64::MAX;
        let mut own = 0u64;
        let mut stab = 0u32;
        for g in 0..6 {
            let c = &self.conj[g];
            let mut k = 0u64;
            for &x in v.iter() {
                k = k * 6 + u64::from(c[x as usize]);
            }
            if g == 0 {
                own = k;
            }
            if k == own {
                stab += 1;
            }
            if k < best {
                best = k;
            }
        }
        (best, stab)
    }

    pub fn apply_sym(&self, s: usize, u: &Cfg) -> Cfg {
        let mut o = [0u8; NL];
        for (t, &(src, iv)) in self.sym[s].iter().enumerate() {
            let x = u[src as usize];
            o[t] = if iv { self.inv[x as usize] } else { x };
        }
        o
    }

    /// Class key = minimum orbit key over the 48 lattice symmetries.
    pub fn class_key(&self, u: &Cfg) -> u64 {
        let mut best = u64::MAX;
        for s in 0..48 {
            let k = self.orbit_key(&self.apply_sym(s, u)).0;
            if k < best {
                best = k;
            }
        }
        best
    }

    /// (class key, class size, orbit weight w = 6 / stabiliser)
    pub fn class_info(&self, u: &Cfg) -> (u64, u32, u32) {
        let mut keys = [0u64; 48];
        for (s, k) in keys.iter_mut().enumerate() {
            *k = self.orbit_key(&self.apply_sym(s, u)).0;
        }
        keys.sort_unstable();
        let mut n = 1u32;
        for i in 1..48 {
            if keys[i] != keys[i - 1] {
                n += 1;
            }
        }
        let stab = self.orbit_key(u).1;
        (keys[0], n, 6 / stab)
    }

    /// Configuration in tree gauge with the given key digits on the free links.
    pub fn cfg_from_key(&self, key: u64) -> Cfg {
        let mut u = [0u8; NL];
        for (i, &(_, l, _)) in self.free.iter().enumerate() {
            u[l] = ((key / self.pow6[i]) % 6) as u8;
        }
        u
    }

    /// Move number mv = 3*link + (t-1), t = 1,2,3 the transposition: U_link -> t * U_link.
    #[inline]
    pub fn apply_move(&self, u: &Cfg, mv: usize) -> Cfg {
        let mut o = *u;
        let l = mv / 3;
        o[l] = self.m((mv % 3 + 1) as u8, o[l]);
        o
    }

    /// Gauge transform: U_(a->b) -> g_a U g_b^-1.
    pub fn apply_gauge(&self, u: &Cfg, g: &[u8; NS]) -> Cfg {
        let mut o = [0u8; NL];
        for y in 0..4 {
            for x in 0..3 {
                let a = site(x, y);
                o[lx(x, y)] = self.m(self.m(g[a], u[lx(x, y)]), self.inv[g[site(x + 1, y)] as usize]);
                o[ly(x, y)] = self.m(self.m(g[a], u[ly(x, y)]), self.inv[g[site(x, y + 1)] as usize]);
            }
        }
        o
    }
}
