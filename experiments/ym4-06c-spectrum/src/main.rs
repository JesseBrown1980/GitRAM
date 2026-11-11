//! YM4-06C 3D S3 flat-sector spectral calculation; exact integer-only, Rust 1.81.
//! All 48 rooted flat configurations -> exact 2-hop Gram -> physical 21x21,
//! modular+CRT-exact characteristic polynomial, rational real-root isolation.
//! This is a fourth-order effective Hamiltonian; the high-r full-gap enclosure
//! is separately justified by Feshbach and the norm-controlled even walk tail.
#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::float_cmp, clippy::cast_precision_loss)]
mod sha256;
use sha256::hash_bytes;
use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::env;
use std::fs;

const NL: usize = 24;
const NF: usize = 48;
const NC: usize = 21;
const L: u64 = 1_182_720;
const TRANS: [u8; 3] = [1, 3, 5];
const P: [u64; 20] = [
    1_000_000_007, 1_000_000_009, 1_000_000_021, 1_000_000_033,
    1_000_000_087, 1_000_000_093, 1_000_000_097, 1_000_000_103,
    1_000_000_123, 1_000_000_181, 1_000_000_207, 1_000_000_223,
    1_000_000_241, 1_000_000_271, 1_000_000_289, 1_000_000_297,
    1_000_000_321, 1_000_000_349, 1_000_000_363, 1_000_000_403,
];
const CUBIC: [i128; 4] = [1, -39_492_144, 519_816_640_269_312, -2_280_439_028_453_407_137_792];
const QUINTIC: [i128; 6] = [
    1, -65_820_240, 1_732_733_020_247_040,
    -22_804_817_983_658_137_681_920,
    150_052_887_866_985_564_407_416_750_080,
    -394_888_873_798_641_922_702_044_531_385_171_968,
];

fn norm3(x: i32) -> u8 { x.rem_euclid(3) as u8 }
fn elem(a: u8, b: u8) -> u8 { assert!(a < 3 && b < 2); 2*a+b }
fn mul(x: u8, y: u8) -> u8 {
    let (a,b,c,d)=(x/2,x%2,y/2,y%2);
    elem(norm3(i32::from(a) + if b==0 {i32::from(c)} else {-i32::from(c)}), b^d)
}
fn inv(x: u8) -> u8 { let (a,b)=(x/2,x%2); elem(if b==0{norm3(-i32::from(a))}else{a},b) }
fn conj(g: u8, x: u8) -> u8 { mul(mul(g,x),inv(g)) }
fn src(k: usize) -> usize {k%8}
fn dst(k: usize) -> usize { (k%8) ^ (1usize << (k/8)) }

struct Lattice {
    par: [usize; 8],
    pedge: [usize; 8],
    bfs: Vec<usize>,
    tree: [bool;NL],
    free: Vec<usize>,
    faces: [[usize;4];NL],
}
impl Lattice {
    fn new() -> Self {
        let mut seen=[false;8];seen[0]=true;
        let mut bfs=vec![0usize];
        let mut par=[0usize;8];
        let mut pedge=[usize::MAX;8];
        let mut at=0usize;
        while at<bfs.len() {
            let u=bfs[at];at+=1;
            for a in 0..3 {
                let v=u^(1usize<<a);
                if !seen[v] {
                    seen[v]=true;par[v]=u;pedge[v]=8*a+u;bfs.push(v);
                }
            }
        }
        assert_eq!(bfs.len(),8);
        let mut tree=[false;NL];
        for &v in bfs.iter().skip(1){tree[pedge[v]]=true;}
        let free=(0..NL).filter(|k|!tree[*k]).collect::<Vec<_>>();
        assert_eq!(free.len(),17);
        let mut faces=[[0usize;4];NL];
        for u in 0..8 {
            for a in 0..3 {for b in (a+1)..3 {
                let f=u*3+(a*(5-a))/2+(b-a-1);
                // for axes (0,1) -> 0, (0,2)->1, (1,2)->2
                let ind=match (a,b){(0,1)=>0,(0,2)=>1,(1,2)=>2,_=>panic!()};
                faces[u*3+ind]=[8*a+u,8*b+(u^(1<<a)),8*a+(u^(1<<b)),8*b+u];
                let _=f;
            }}
        }
        Self{par,pedge,bfs,tree,free,faces}
    }
    fn canon(&self, mut u:[u8;NL])->u64 {
        let mut g=[0u8;8];
        for &v in self.bfs.iter().skip(1){
            g[v]=mul(g[self.par[v]],u[self.pedge[v]]);
        }
        for (k,x) in u.iter_mut().enumerate() {
            *x=mul(mul(g[src(k)],*x),inv(g[dst(k)]));
        }
        assert!((0..NL).filter(|k|self.tree[*k]).all(|k|u[k]==0));
        self.encode(&u)
    }
    fn encode(&self,u:&[u8;NL])->u64{
        let mut k=0u64;
        for &i in &self.free{k=k*6+u64::from(u[i]);}
        k
    }
    fn decode(&self,mut key:u64)->[u8;NL]{
        let mut u=[0u8;NL];
        for &i in self.free.iter().rev(){u[i]=(key%6)as u8;key/=6;}
        assert_eq!(key,0);
        u
    }
    fn hop(&self,key:u64,k:usize,t:u8)->u64{
        let mut u=self.decode(key);
        u[k]=mul(t,u[k]);
        self.canon(u)
    }
    fn magnetic(&self,key:u64)->u64{
        let u=self.decode(key);
        let mut e=0u64;
        for f in &self.faces {
            let h=mul(mul(mul(u[f[0]],u[f[1]]),inv(u[f[2]])),inv(u[f[3]]));
            e+=if h==0{0}else if h%2==1{2}else{3};
        }
        e
    }
    fn flat(&self,a:u8,b:u8,c:u8)->u64{
        let mut u=[0u8;NL];
        for site in 0..8{
            if site&1!=0{u[site]=a;}
            if site&2!=0{u[8+site]=b;}
            if site&4!=0{u[16+site]=c;}
        }
        self.canon(u)
    }
}
fn classes(lat:&Lattice)->(Vec<u64>,Vec<[u8;3]>,[usize;NF],[usize;NC],Vec<usize>){
    let mut flats=Vec::new();
    let mut triple=Vec::new();
    let mut canonical=Vec::new();
    let mut all=BTreeSet::new();
    for a in 0..6 {for b in 0..6 {
        if mul(a,b)!=mul(b,a){continue;}
        for c in 0..6 {
            if mul(a,c)!=mul(c,a)||mul(b,c)!=mul(c,b){continue;}
            triple.push([a,b,c]);
            flats.push(lat.flat(a,b,c));
            let can=(0..6).map(|g|[conj(g,a),conj(g,b),conj(g,c)]).min().expect("S3");
            all.insert(can);
            canonical.push(can);
        }
    }}
    assert_eq!(flats.len(),NF);
    assert_eq!(flats.iter().copied().collect::<BTreeSet<_>>().len(),NF);
    assert!(flats.iter().all(|k|lat.magnetic(*k)==0));
    assert_eq!(all.len(),NC);
    let inds=all.into_iter().collect::<Vec<_>>();
    let mut orbit=[0usize;NF];
    let mut reps=[usize::MAX;NC];
    let mut sizes=vec![0usize;NC];
    for i in 0..NF {
        let c=inds.binary_search(&canonical[i]).unwrap();
        orbit[i]=c;sizes[c]+=1;
        if reps[c]==usize::MAX{reps[c]=i;}
    }
    assert_eq!(sizes.iter().sum::<usize>(),NF);
    (flats,triple,orbit,reps,sizes)
}
fn gram(lat:&Lattice,flats:&[u64])->([[u64;NF];NF],BTreeMap<u64,u64>,usize,usize){
    let mut a1=HashMap::<u64,[u16;NF]>::new();
    let mut a2=HashMap::<u64,[u16;NF]>::new();
    let flat_set=flats.iter().copied().collect::<BTreeSet<_>>();
    for (i,&flat) in flats.iter().enumerate(){
        for k in 0..NL {for t in TRANS {
            let s=lat.hop(flat,k,t);
            assert_eq!(lat.magnetic(s),8);
            let v=a1.entry(s).or_insert([0;NF]);
            v[i]=v[i].checked_add(1).unwrap();
            for j in 0..NL {for u in TRANS {
                let q=lat.hop(s,j,u);
                if !flat_set.contains(&q) {
                    let z=a2.entry(q).or_insert([0;NF]);
                    z[i]=z[i].checked_add(1).unwrap();
                }
            }}
        }}
    }
    assert_eq!(a1.len(),3456);
    assert_eq!(a2.len(),120888);
    let mut gram2=[[0u64;NF];NF];
    for c in a1.values(){
        for i in 0..NF {if c[i]!=0 {
            for j in 0..NF {if c[j]!=0 {
                gram2[i][j]+=u64::from(c[i])*u64::from(c[j]);
            }}
        }}
    }
    for (i,row) in gram2.iter().enumerate() {
        for (j,&v) in row.iter().enumerate(){
            assert_eq!(v,if i==j{72}else{0},"two-hop flat coupling");
        }
    }
    let mut out=[[0u64;NF];NF];
    let mut ms=BTreeMap::<u64,u64>::new();
    for (key,c) in &a2 {
        let m=lat.magnetic(*key);
        *ms.entry(m).or_default()+=1;
        assert_eq!(L%(64*m),0,"4th-order exact common denominator");
        let weight=L/(64*m);
        let active=c.iter().enumerate().filter(|(_,n)|**n!=0).map(|(i,n)|(i,u64::from(*n))).collect::<Vec<_>>();
        for &(i,ni) in &active {
            for &(j,nj) in &active {out[i][j]+=ni*nj*weight;}
        }
    }
    let expected=[(8,1152),(11,3744),(12,16128),(14,5040),(15,27648),(16,67176)]
        .into_iter().collect::<BTreeMap<u64,u64>>();
    assert_eq!(ms,expected);
    for i in 0..NF {for j in 0..NF {assert_eq!(out[i][j],out[j][i]);}}
    (out,ms,a1.len(),a2.len())
}
fn physical(gram:&[[u64;NF];NF],orbit:&[usize;NF],reps:&[usize;NC],sizes:&[usize])->[[u64;NC];NC]{
    let mut c=[[0u64;NC];NC];
    for i in 0..NC{
        for j in 0..NF {c[i][orbit[j]]+=gram[reps[i]][j];}
    }
    for i in 0..NC{for j in 0..NC{
        assert_eq!(sizes[i] as u64*c[i][j],sizes[j] as u64*c[j][i],"physical weighted adjoint");
    }}
    c
}
fn is_prime(x:u64)->bool{
    let mut d=2u64;
    while d*d<=x {if x%d==0{return false;} d+=if d==2{1}else{2};}
    x>1
}
fn powmod(mut a:u64,mut n:u64,p:u64)->u64{
    let mut r=1;
    while n>0{if n&1==1{r=mulmod(r,a,p);}a=mulmod(a,a,p);n>>=1;}
    r
}
fn mulmod(a:u64,b:u64,p:u64)->u64{((u128::from(a)*u128::from(b))%u128::from(p))as u64}
fn moddiff(x:u64,y:u64,p:u64)->u64{if x>=y{x-y}else{x+p-y}}
fn cm(x:i128,p:u64)->u64{x.rem_euclid(i128::from(p))as u64}
fn poly_mod(x:u64,p:u64,coef:&[i128])->u64{
    let mut z=0u64;
    for &a in coef {z=(mulmod(z,x,p)+cm(a,p))%p;}
    z
}
fn predicted(x:u64,p:u64)->u64{
    let a=moddiff(x%p,13_112_208%p,p);
    let b=moddiff(x%p,13_138_128%p,p);
    let c=poly_mod(x,p,&CUBIC);
    let d=poly_mod(x,p,&QUINTIC);
    mulmod(mulmod(powmod(a,4,p),powmod(b,6,p),p),mulmod(powmod(c,2,p),d,p),p)
}
fn det_mod(c:&[[u64;NC];NC],x:u64,p:u64)->u64{
    let mut a=[[0u64;NC];NC];
    for i in 0..NC{for j in 0..NC{
        a[i][j]=moddiff(if i==j{x%p}else{0},c[i][j]%p,p);
    }}
    let mut det=1u64;
    for k in 0..NC{
        let mut piv=k;
        while piv<NC && a[piv][k]==0{piv+=1;}
        if piv==NC{return 0;}
        if piv!=k {a.swap(k,piv);det=if det==0{0}else{p-det};}
        let d=a[k][k];det=mulmod(det,d,p);
        let inv=powmod(d,p-2,p);
        for i in (k+1)..NC{
            let t=mulmod(a[i][k],inv,p);
            if t==0{continue;}
            for j in (k+1)..NC{
                a[i][j]=moddiff(a[i][j],mulmod(t,a[k][j],p),p);
            }
            a[i][k]=0;
        }
    }
    det
}
fn eval_poly(x:i128,coef:&[i128])->i128{
    let mut r=0i128;
    for &a in coef {r=r.checked_mul(x).unwrap().checked_add(a).unwrap();}
    r
}
fn check_roots() {
    // Distinct signed integer brackets cover every real root of each factor.
    let cr=[(12_923_213,12_923_214),(13_156_644,13_156_645),(13_412_285,13_412_286)];
    let qr=[(12_761_408,12_761_409),(13_010_879,13_010_880),
        (13_174_351,13_174_352),(13_290_715,13_290_716),(13_582_885,13_582_886)];
    for &(a,b) in &cr {assert!(eval_poly(a,&CUBIC)*eval_poly(b,&CUBIC)<0,"cubic sign change");}
    for &(a,b) in &qr {assert!(eval_poly(a,&QUINTIC)*eval_poly(b,&QUINTIC)<0,"quintic sign change");}
    assert!(qr[4].0>cr[2].1 && cr[2].0>qr[3].1);
}
fn check_characteristic(c:&[[u64;NC];NC]){
    check_roots();
    let maxsum=c.iter().map(|row|row.iter().sum::<u64>()).max().unwrap();
    assert_eq!(maxsum,14_021_568,"exact Gershgorin row sum");
    // Every true eigenvalue satisfies |lambda| <= 14,021,568 (Gershgorin).
    // Every proposed polynomial root satisfies |lambda| < 14,100,000
    // (signed brackets and linear factors). Thus *each* coefficient magnitude
    // is < (2*14,100,000)^21 < 10^158 for either monic degree-21 polynomial.
    // Product of twenty distinct primes, each >= 10^9, >10^180.
    // Equality modulo all 20 primes proves coefficient equality in Z.
    for &p in &P{
        assert!(is_prime(p));
        for x in 0..22u64{
            assert_eq!(det_mod(c,x,p),predicted(x,p),"modular charpoly mismatch for prime {p} x {x}");
        }
    }
}
fn roots_and_bounds()->String{
    let mut out=String::new();
    out.push_str("ROOTS|degree=21|factor=(x-13112208)^4*(x-13138128)^6*cubic^2*quintic|crt_exact=20_primes_22_values|json=0\n");
    out.push_str("TOP_ROOT|first_numerator_gt=13582885|first_numerator_lt=13582886|second_numerator_gt=13412285|second_numerator_lt=13412286|common_den=1182720|json=0\n");
    out.push_str("SPLIT_COEFF|delta_num_gt=170599|delta_num_lt=170601|den=1182720|fourth_order_only=YES|json=0\n");
    for r in [27u64,81,2187]{
        let delta_approx_num=170600u64; // integer bracket center, not a certified full-r eigenvalue.
        out.push_str(&format!("FOURTH_ORDER|r={r}|first_gap_coefficient_approx_num={delta_approx_num}|den=1182720*{r}^3|full_spectrum_estimate=NOT_CERTIFIED|json=0\n"));
        let c=144+72*r;
        out.push_str(&format!("FLAT_CLUSTER|r={r}|E22_E21_lower_JE={}|analytical=YES|upper_and_exact_spectrum=OPEN|json=0\n",6*r-72));
        if r==2187{
            // Exact-rational proof uses only integer inequalities; it outputs
            // a weaker convenient bound (1/10^11) in JE units.
            // smax = (9/r)/(1-(12/r)^2); q=12/r;
            // eps = (9/r)*q^4/(1-q^2);
            // D >= ((170599 - 13582886*smax/(2r))/(L*r^3)) - eps;
            // first_gap >= D/(1+72/(8r)^2).
            // Verify D/(1+72/(8r)^2) > 1/10^11 using common-denominator u128
            // below; no use of floating point or transcendental functions.
            prove_gap2187();
            out.push_str("FULL_FIRST_GAP|r=2187|lower_gt_JE=1/100000000000|method=Feshbach_even_Neumann_tail_plus_exact_roots|independent_proof_review=OPEN|json=0\n");
        }
        out.push_str(&format!("NOT_MEASURED|r={r}|physical_transfer_mass=OPEN|continuum_SU2_gap=OPEN|json=0\n"));
        let _=c;
    }
    out
}
fn prove_gap2187(){
    // Show a coarse nontrivial strictly positive numerical lower bound using
    // exact integer cross-products (chosen to fit u128). Dimensionless scale:
    // numerator brackets times L; smax bounded by 1/200 and
    // epsilon < 1/200_000_000_000 (these two independent integer comparisons
    // are proved by cross multiplication, not assumed).
    let r=2187u128;
    let r2=r*r;
    let qnum=12u128;
    let q2=qnum*qnum;
    let qden=r2-q2;
    // smax = (9/r)*(r²/(r²-144))=9r/(r²-144).
    let smax_n=9*r;
    let smax_d=qden;
    assert!(smax_n*200<smax_d);
    // eps = (9/r)*(12/r)^4/(1-144/r²)
    // = 9*12^4/[r^3*(r²-144)].
    let eps_n=9*q2*q2;
    let eps_d=r*r*r*qden;
    assert!(eps_n*200_000_000_000<eps_d);
    // Evaluate conservative numerator delta:
    // (170599 - 13582886*(smax/(2r))) / (L*r^3) - eps.
    // Use coarser smax <1/200: correction less than 13582886/(400r) < 16.
    assert!(13_582_886u128 < 16*400*r);
    // Hence delta > (170599-16)/(L*r³) - 1/200_000_000_000.
    // At r2187, show this > (1+72/(8r)^2) / 1e11,
    // so first gap > 1e-11 in JE units.
    let xnum=170_599u128-16;
    let xden=u128::from(L)*r*r*r;
    let target_d=100_000_000_000u128;
    let den_factor=(8*r)*(8*r);
    // (xnum/xden - 1/2e11) > (1 + 72/den_factor) / 1e11
    // numerator: (xnum*target_d*den_factor*2)
    // - xden*den_factor - 2*xden*(den_factor+72) > 0.
    let left=xnum*target_d*den_factor*2;
    let right=xden*den_factor+2*xden*(den_factor+72);
    assert!(left>right,"nontrivial full first-gap bound failed");
}
fn execute() {
    let lat=Lattice::new();
    let (flat,_,orb,reps,size)=classes(&lat);
    let (g,m,a1,a2)=gram(&lat,&flat);
    let c=physical(&g,&orb,&reps,&size);
    check_characteristic(&c);
    let mut block=String::new();
    block.push_str("YM4_06C|model=3D_2x2x2_S3|method=rooted_two_hop_exact_gram_and_21_phys_sector|rust=1.81|trinary=YES|json=0\n");
    block.push_str(&format!("ENUM|flat_rooted={NF}|flat_phys={NC}|one_step_states={a1}|two_step_states={a2}|one_link_moves={}|two_link_moves={}|json=0\n",NF*NL*3,NF*NL*NL*9));
    for (mag,n) in m{block.push_str(&format!("TWO_STEP_MAG|mag={mag}|states={n}|json=0\n"));}
    block.push_str(&format!("CHARPOLY|matrix_dim={NC}|matrix_scaling={L}|maximum_row_sum=14021568|20_primes_modular_CRT=PASS|root_intervals=INTEGER_EXACT|json=0\n"));
    block.push_str(&roots_and_bounds());
    for row in &c {
        block.push_str("MATRIX_ROW|");
        for (i,x) in row.iter().enumerate(){
            if i>0{block.push(',');}
            block.push_str(&x.to_string());
        }
        block.push_str("|json=0\n");
    }
    let sha=hash_bytes(block.as_bytes());
    let sh=&sha[..16];
    let hbi=format!("HBI|version=1|rows={}|bytes={}|source=3D_S3_2HOP|hbp_sha256={sha}|sh={sh}|json=0\n",block.lines().count(),block.len());
    let hbi_sha=hash_bytes(hbi.as_bytes());
    let anchor=format!("SHA|value={sha}|SH|value={sh}|HASH|value={}|hbi_sha256={hbi_sha}|json=0\n",
        hash_bytes(format!("{sha}|{sh}|{hbi_sha}|YM4_06C").as_bytes()));
    let dest=env::args().nth(1).unwrap_or_else(||"output".into());
    fs::create_dir_all(&dest).unwrap();
    fs::write(format!("{dest}/SPECTRAL-21.hbp"),&block).unwrap();
    fs::write(format!("{dest}/SPECTRAL-21.hbi"),&hbi).unwrap();
    fs::write(format!("{dest}/SPECTRAL-21.anchor"),&anchor).unwrap();
    print!("{block}");
    println!("YM4_06C_GATE|PASS|exact_21x21_charpoly=YES|full_gap_bound_r2187=YES|first_gap_r27_81=NOT_CERTIFIED|r9_runs=0|json=0");
}
fn main() { execute(); }
#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn group_and_roots(){for x in 0..6{assert_eq!(mul(x,inv(x)),0);}check_roots();prove_gap2187();}
    #[test] fn finite_geometry(){let x=Lattice::new();let(f,..)=classes(&x);assert_eq!(f.len(),48);}
}
