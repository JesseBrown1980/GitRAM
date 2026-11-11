// Stage B engine to Stage A transport adapter.
// Imports actual immutable Stage A lat.rs; no code from Stage A is modified.
// Not an independently authored Hamiltonian implementation, and no coupling.
#[path = "../cleanroom/src/lat.rs"]
mod lat;
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

const ENG_WORDS: [&str; 6] = ["012", "021", "102", "120", "201", "210"];
fn stage_code(s: &str) -> Option<u8> {
    match s {
        "012" => Some(0), "102" => Some(1), "210" => Some(2),
        "021" => Some(3), "120" => Some(4), "201" => Some(5), _ => None,
    }
}
fn parse_engine(line: &str) -> Result<(u64, [u8; 24]), String> {
    let mut parts = line.trim_end().split('|');
    if parts.next() != Some("E") { return Err("missing E tag".into()); }
    let key: u64 = parts.next().and_then(|s| s.strip_prefix("key="))
        .ok_or("missing key")?.parse().map_err(|_| "invalid key")?;
    let wordstr = parts.next().and_then(|s| s.strip_prefix("links=")).ok_or("missing links")?;
    if parts.next().is_some() { return Err("extra fields".into()); }
    let words = wordstr.split(',').collect::<Vec<_>>();
    if words.len() != 24 { return Err(format!("wrong number of links {}", words.len())); }
    let free: [usize; 13] = [8, 9, 10, 11, 15, 16, 17, 18, 19, 20, 21, 22, 23];
    let mut rebuilt = 0u64;
    let mut cfg = [0u8; 24];
    for (slot, word) in words.iter().enumerate() {
        let k = ENG_WORDS.iter().position(|x| x == word).ok_or("invalid word")?;
        if free.contains(&slot) {
            rebuilt = rebuilt.checked_mul(6).and_then(|x| x.checked_add(k as u64)).ok_or("key overflow")?;
        } else if k != 0 { return Err("nonidentity tree link".into()); }
        let physical = if slot < 12 { slot } else { slot - 12 };
        let (x, y) = (physical / 4, physical % 4);
        let d = if slot < 12 { 0 } else { 1 };
        cfg[2 * (x + 3 * y) + d] = stage_code(word).ok_or("bad stage word")?;
    }
    if rebuilt != key { return Err(format!("engine key mismatch {} {}", rebuilt, key)); }
    Ok((key, cfg))
}
fn cfg_digits(cfg: &[u8;24]) -> String {
    cfg.iter().map(|x| char::from(b'0'+*x)).collect()
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let known = std::env::args().nth(1).as_deref() == Some("--known");
    let ctx = lat::Ctx::new(0);
    let mut out = io::BufWriter::new(io::stdout());
    let mut seen_key = HashSet::new();
    let mut seen_stage = HashSet::new();
    let mut count = 0u64;
    for line in io::stdin().lock().lines() {
        let line = line?;
        let (key, cfg) = parse_engine(&line)?;
        if !seen_key.insert(key) { return Err("duplicate input engine key".into()); }
        let m = ctx.mag_total(&cfg);
        let (stage_key, size, w) = ctx.class_info(&cfg);
        if known {
            writeln!(out,"KAT|engine_key={}|stage_key={}|M={}|size={}|W={}|cfg={}|json=0",
                     key,stage_key,m,size,size*w,cfg_digits(&cfg))?;
        } else {
            if m == 0 { return Err("flat key in core".into()); }
            if !seen_stage.insert(stage_key) { return Err(format!("collision for stage key {}", stage_key).into()); }
            writeln!(out,"MAP|engine_key={}|stage_key={}|M={}|size={}|W={}|json=0",
                     key,stage_key,m,size,size*w)?;
        }
        count += 1;
    }
    out.flush()?;
    eprintln!("LOOKUP|PASS|records={}|collisions=0|coupling=none",count);
    Ok(())
}
