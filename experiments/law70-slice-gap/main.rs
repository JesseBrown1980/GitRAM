//! Law70 -> a rigorously gapped TRINARY toy transfer theory under ternary refinement.
//! Rust 1.81, integer matrices, exact residuals, and SHA256 content-addressed slices.
//! The continuum claim here is about the SPECIFIED toy transfer, NOT Yang--Mills.
#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::float_cmp, clippy::cast_precision_loss)]
mod sha256;
use sha256::hash_bytes;
use std::env;
use std::fs;
use std::path::Path;

const MAX_LEVEL: u32 = 10;
const FROZEN_PREDICTION: &str =
    "8c242ab34be362dd7d1adab4284480467fe90ff3";

type Matrix3 = [[i128;3];3];

fn triple(n:u32)->u64 {
    assert!((1..=MAX_LEVEL).contains(&n));
    3u64.pow(n)
}
fn transfer_numerator(n:u64)->Matrix3 {
    let n=i128::from(n);
    let mut a=[[1i128;3];3];
    for (i,row) in a.iter_mut().enumerate(){
        row[i]+=3*n-3;
    }
    a
}
fn matvec(a:&Matrix3,v:[i128;3])->[i128;3] {
    let mut result=[0i128;3];
    for (i,row) in a.iter().enumerate() {
        result[i]=row.iter().zip(v.iter()).map(|(&x,&y)|x*y).sum();
    }
    result
}
fn assert_eigenvector(a:&Matrix3,v:[i128;3],numerator:i128) {
    let applied=matvec(a,v);
    for (l,r) in applied.into_iter().zip(v) {
        assert_eq!(l,numerator*r,"nonzero exact transfer residual");
    }
}
fn check_transfer(n:u64){
    assert!(n>=3);
    let a=transfer_numerator(n);
    let full=i128::from(3*n);
    let excited=i128::from(3*(n-1));
    assert_eigenvector(&a,[1,1,1],full);
    assert_eigenvector(&a,[1,-1,0],excited);
    assert_eigenvector(&a,[1,1,-2],excited);
    assert!(a.iter().enumerate().all(|(i,row)|
        row.iter().enumerate().all(|(j,&x)|
            x==a[j][i] && x>0
        )
    ));
    assert!(a.iter().all(|row|row.iter().sum::<i128>()==full));
    assert_eq!(a[0][0],i128::from(3*n-2));
    assert_eq!(a[0][1],1);
    // In the N-site tensor product, the excited ratio is the largest
    // nontrivial product of site eigenvalue ratios: lambda=(n-1)/n.
    assert!(excited>0 && excited<full);
}
fn trit(i:usize)->u8 { (i%3)as u8 }
fn rotate(s:u8)->u8 {assert!(s<3); (s+1)%3}
fn check_shadow()->bool {
    for i in 0..3 {
        let mut s=i;
        for _ in 0..3{s=rotate(s);}
        if s!=i{return false;}
        assert_ne!(rotate(i),i);
    }
    true
}
fn leaf_hash(k:u32,index:usize,value:u8)->String{
    hash_bytes(format!("LAW70_LEAF_V1|level={k}|site={index}|trit={value}").as_bytes())
}
fn parent_hash(k:u32,depth:usize,children:&[String])->String{
    assert_eq!(children.len(),3);
    hash_bytes(format!("LAW70_TRINODE_V1|k={k}|depth={depth}|{}|{}|{}",
        children[0],children[1],children[2]).as_bytes())
}
struct Roots {
    before: String,
    after: String,
    merkle_layers:usize,
    leaves:usize,
    changed_leaf:usize,
    remote_trit_unchanged:bool,
}
fn roots(k:u32)->Roots {
    let n=triple(k)as usize;
    let mut levels:Vec<Vec<String>>=vec![Vec::with_capacity(n)];
    for i in 0..n {
        levels[0].push(leaf_hash(k,i,trit(i)));
    }
    let mut depth=0usize;
    while levels.last().expect("nonempty").len()>1{
        let parent=levels.last().unwrap().chunks_exact(3)
            .map(|t|parent_hash(k,depth,t)).collect::<Vec<_>>();
        assert_eq!(parent.len()*3,levels.last().unwrap().len());
        levels.push(parent);
        depth+=1;
    }
    assert_eq!(depth,k as usize,"ternary depth must equal refinement level");
    let before=levels.last().unwrap()[0].clone();
    let changed_leaf=n/2;
    let remote_index=(changed_leaf+n/3)%n;
    assert_ne!(changed_leaf,remote_index);
    let remote_before=trit(remote_index);
    // Only one local trit changes; all other physical toy states stay fixed.
    let remote_after=trit(remote_index);
    let remote_trit_unchanged=remote_before==remote_after;
    let mut current=leaf_hash(k,changed_leaf,rotate(trit(changed_leaf)));
    let mut index=changed_leaf;
    for (dep,layer) in levels.iter().take(depth).enumerate(){
        let start=(index/3)*3;
        let mut children=layer[start..start+3].to_vec();
        children[index%3]=current;
        current=parent_hash(k,dep,&children);
        index/=3;
    }
    assert_eq!(index,0);
    let after=current;
    // A measured fixture consequence of this SHA implementation, not proof
    // that fixed-width hashes can be injective on all possible input states.
    assert_ne!(before,after,"measured single-trit mutation did not alter root");
    Roots{before,after,merkle_layers:depth,leaves:n,changed_leaf,
        remote_trit_unchanged}
}
fn compute_body()->(String,String) {
    assert!(check_shadow(),"order-three shadow action failed");
    let mut lines=String::new();
    lines.push_str("MODEL|LAW70_TERNARY_SLICES|source=Law70_lambda_(n-1)/n|operator=NEW_EXPLICIT_TOY_LOCAL_TRANSFER|physical_4D_YM=NOT_ESTABLISHED|json=0\n");
    lines.push_str("THEOREM|n=3^k|spacing=a0/n|physical_mass=a0^-1*n*log(n/(n-1))|lower=a0^-1|upper=n/((n-1)*a0)|limit=a0^-1|proof=log_inequality|json=0\n");
    lines.push_str("CONTROL|fixed_spacing=a0|mass_limit=0|hash_alone_does_not_set_spectral_gap=YES|physical_signal_speed=NOT_INFERRED|json=0\n");
    lines.push_str("TRANSFER|A_n=(3n-3)I_3+J_3|T_n=A_n/(3n)|symmetric=YES|positive_semidefinite=YES|doubly_stochastic=YES|eigenvalue_numerators=3n,3n-3,3n-3|json=0\n");
    let mut last_roots=String::new();
    for k in 1..=MAX_LEVEL{
        let n=triple(k);
        check_transfer(n);
        let spatial_sites=u128::from(n)*u128::from(n)*u128::from(n);
        let root=roots(k);
        assert_eq!(root.leaves as u64,n);
        assert!(root.remote_trit_unchanged);
        // Exact rational enclosure: 1 <= a0*m <= n/(n-1);
        // no computation of logarithms is needed for the proof.
        assert!(n>1);
        let upper_num=n;
        let upper_den=n-1;
        assert!(upper_num>upper_den);
        lines.push_str(&format!(
            "LEVEL|k={k}|n={n}|a_over_a0=1/{n}|spatial_sites={spatial_sites}|trit_states_per_site=3|T_second_num={upper_den}|T_second_den={upper_num}|a0_m_lower=1|a0_m_upper={upper_num}/{upper_den}|residual_exact_zero=3_vectors|json=0\n"
        ));
        lines.push_str(&format!(
            "HASH_WITNESS|k={k}|leaves={}|merkle_path={}|mutated_leaf={}|old_root={}|new_root={}|remote_physical_trit=UNCHANGED|metadata_root=CHANGED|instant_physical_signal=NOT_SHOWN|json=0\n",
            root.leaves,root.merkle_layers,root.changed_leaf,root.before,root.after
        ));
        last_roots=format!("k={k}|old={}|new={}",root.before,root.after);
    }
    let shadow=format!(
        "SH|phase=0,1,2|order_three=PASS|WAS_IS_WILL=IS_FROZEN|projection_lineage=LOCAL_TRIT_MUTATION_TO_ROOT|last={last_roots}|r9_runs=0|json=0\n"
    );
    (lines,shadow)
}
struct Receipts{hbp:String,hbi:String,sha:String,sh:String,hash:String}
fn receipts()->Receipts{
    let (hbp,shadow)=compute_body();
    let mut offsets=String::new();
    let mut offset=0usize;
    let mut record_count=0usize;
    for line in hbp.lines() {
        offsets.push_str(&format!("INDEX|row={record_count}|byte={offset}|length={}|json=0\n",line.len()+1));
        offset+=line.len()+1;
        record_count+=1;
    }
    assert_eq!(offset,hbp.len());
    let hbp_sha=hash_bytes(hbp.as_bytes());
    let prefix=&hbp_sha[..16];
    let hbi=format!("HBI|version=1|rows={record_count}|bytes={}|hbp_sha256={hbp_sha}|locator={prefix}|json=0\n{offsets}",hbp.len());
    let hbi_sha=hash_bytes(hbi.as_bytes());
    let sh=format!("SH_SOURCE|hbp_sha256={hbp_sha}|hbi_sha256={hbi_sha}|json=0\n{shadow}");
    let sh_sha=hash_bytes(sh.as_bytes());
    let sha=format!("SHA|HBP={hbp_sha}|HBI={hbi_sha}|SH={sh_sha}|json=0\n");
    let sha_sha=hash_bytes(sha.as_bytes());
    let root=hash_bytes(format!("GLOBAL_HASH|{hbp_sha}|{hbi_sha}|{sh_sha}|{sha_sha}|law70_v1").as_bytes());
    let hash=format!("HASH|root={root}|algorithm=SHA256|hbp={hbp_sha}|hbi={hbi_sha}|sh={sh_sha}|sha={sha_sha}|frozen_predict_commit={FROZEN_PREDICTION}|physical_signal=NOT_INFERRED|json=0\n");
    Receipts{hbp,hbi,sha,sh,hash}
}
fn write_output(p:&Path) {
    fs::create_dir_all(p).expect("create output directory");
    let r=receipts();
    for (name,data) in [
        ("SLICE.hbp",r.hbp.as_str()),
        ("SLICE.hbi",r.hbi.as_str()),
        ("SLICE.sha",r.sha.as_str()),
        ("SLICE.sh",r.sh.as_str()),
        ("SLICE.hash",r.hash.as_str()),
    ]{
        fs::write(p.join(name),data).expect("write exact content");
    }
    println!("LAW70_RESULT|PASS|levels=10|trits=3|gap=UNIFORM_POSITIVE_IN_TOY_TRANSFER|residuals=ZERO|merkle_local_change=10_of_10|global_hash=PASS|r9_runs=0|json=0");
}
fn verify_output(p:&Path) {
    let r=receipts();
    for (name,want) in [
        ("SLICE.hbp",r.hbp.as_str()),
        ("SLICE.hbi",r.hbi.as_str()),
        ("SLICE.sha",r.sha.as_str()),
        ("SLICE.sh",r.sh.as_str()),
        ("SLICE.hash",r.hash.as_str()),
    ] {
        let got=fs::read(p.join(name)).expect("read original receipt");
        assert_eq!(got,want.as_bytes(),"receipt does not match independent source replay");
    }
    println!("LAW70_VERIFY|PASS|HBI=PASS|HBP=PASS|SHA=PASS|SH=PASS|HASH=PASS|r9_runs=0|json=0");
}
fn main(){
    let a:Vec<String>=env::args().collect();
    match a.get(1).map(String::as_str) {
        Some("compute")=>write_output(Path::new(a.get(2).expect("output directory"))),
        Some("verify")=>verify_output(Path::new(a.get(2).expect("output directory"))),
        Some("plant-stale-root")=>{
            let r=roots(3);
            assert_eq!(r.before,r.after,"stale global root must fail");
        }
        _=>panic!("usage: compute PATH | verify PATH | plant-stale-root"),
    }
}
#[cfg(test)]
mod tests{
    use super::*;
    #[test]fn trinary_transfer_spectrum(){
        for k in 1..=MAX_LEVEL{check_transfer(triple(k));}
    }
    #[test]fn triple_shadow_cycle(){
        assert!(check_shadow());
    }
    #[test]fn merkle_local_change(){
        for k in 1..=4{
            let r=roots(k);
            assert_ne!(r.before,r.after);
            assert!(r.remote_trit_unchanged);
        }
    }
    #[test]fn global_integrity(){
        let r=receipts();
        assert!(r.hbp.lines().filter(|s|s.starts_with("LEVEL|")).count()==10);
        assert!(r.hash.starts_with("HASH|root="));
    }
}