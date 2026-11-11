//! Exact 2x2x2 S3 cost-six magnetic support solver.
//! Tree-gauge backtracking with forced trivial-plaquette constraints;
//! independent from the two-hop flat-state walk enumeration.
use super::{conj,inv,mul,Lattice,NL,TRANS};
use std::collections::{BTreeMap,BTreeSet};
const UNKNOWN:u8=255;
fn hol(u:&[u8;NL],f:&[usize;4])->Option<u8>{
    if f.iter().any(|&k|u[k]==UNKNOWN){return None;}
    Some(mul(mul(mul(u[f[0]],u[f[1]]),inv(u[f[2]])),inv(u[f[3]])))
}
fn propagate(u:&mut[u8;NL],skip:&[bool;NL],lat:&Lattice)->bool{
    loop{
        let mut change=false;
        for (id,f) in lat.faces.iter().enumerate(){
            if skip[id]{continue;}
            let missing=f.iter().copied().filter(|&k|u[k]==UNKNOWN).collect::<Vec<_>>();
            match missing.len(){
                0=>{if hol(u,f)!=Some(0){return false;}},
                1=>{
                    let k=missing[0];
                    let mut valid=None;
                    for x in 0..6u8 {
                        u[k]=x;
                        if hol(u,f)==Some(0){
                            if valid.is_some(){return false;}
                            valid=Some(x);
                        }
                    }
                    u[k]=UNKNOWN;
                    if let Some(x)=valid{u[k]=x;change=true;}
                    else{return false;}
                },
                _=>{},
            }
        }
        if !change{return true;}
    }
}
fn visit(mut u:[u8;NL],skip:&[bool;NL],lat:&Lattice,states:&mut BTreeSet<u64>){
    if !propagate(&mut u,skip,lat){return;}
    let free=(0..NL).filter(|&k|u[k]==UNKNOWN).collect::<Vec<_>>();
    if free.is_empty(){
        let k=lat.encode(&u);
        if lat.magnetic(k)==6{states.insert(k);}
        return;
    }
    // Favor a variable in a face with precisely two unknown links.
    let k=free.into_iter().max_by_key(|&x|{
        let mut pairs=0;let mut incidence=0;
        for (fid,f) in lat.faces.iter().enumerate(){
            if skip[fid]||!f.contains(&x){continue;}
            incidence+=1;
            if f.iter().filter(|&&e|u[e]==UNKNOWN).count()==2{pairs+=1;}
        }
        (pairs,incidence)
    }).unwrap();
    for val in 0..6u8{
        let mut v=u;v[k]=val;
        visit(v,skip,lat,states);
    }
}
pub fn certify(lat:&Lattice)->usize{
    let mut states=BTreeSet::new();
    let mut supports=0u64;
    // Periodic length-two tube supports: a pair of equally oriented
    // plaquettes, one in each slice along the remaining axis.
    for plane in 0..3{
        let axis=match plane{0=>2,1=>1,2=>0,_=>panic!()};
        for base in 0..8{
            let across=base^(1<<axis);
            if base>=across{continue;}
            supports+=1;
            let mut skip=[false;NL];
            skip[3*base+plane]=true;
            skip[3*across+plane]=true;
            let mut u=[UNKNOWN;NL];
            for (k,tree) in lat.tree.iter().enumerate(){
                if *tree{u[k]=0;}
            }
            let mut found=BTreeSet::new();
            visit(u,&skip,lat,&mut found);
            assert_eq!(found.len(),18,"each M6 support has 18 rooted solutions");
            states.extend(found);
        }
    }
    assert_eq!(supports,12);
    assert_eq!(states.len(),216);
    let mut classes=BTreeMap::<u64,usize>::new();
    for &key in &states {
        let u=lat.decode(key);
        let canonical=(0..6u8).map(|g|{
            let mut v=u;
            for x in &mut v {*x=conj(g,*x);}
            lat.encode(&v)
        }).min().unwrap();
        *classes.entry(canonical).or_default()+=1;
    }
    assert_eq!(classes.len(),36,"M6 physical gauge orbits");
    assert!(classes.values().all(|&n|n==6),"all M6 conjugation stabilizers must be trivial");
    for &key in &states {
        assert_eq!(lat.magnetic(key),6);
        for link in 0..NL {for t in TRANS {
            assert!(!states.contains(&lat.hop(key,link,t)),
                "magnetic-six states unexpectedly coupled by one electric move");
        }}
    }
    states.len()
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn all_cost_six(){
        assert_eq!(certify(&Lattice::new()),216);
    }
}
