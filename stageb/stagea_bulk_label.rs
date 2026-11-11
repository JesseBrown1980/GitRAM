//! Stage B transport bridge: engine E|key=...|links=... to the
//! unchanged, independently authored Stage A lattice class lookup.
//! Exposes neither engine row internals nor any expected Stage A result.
//! Compile: rustc +1.81.0 --edition=2021 -O stageb/stagea_bulk_label.rs -o bulk
//! Stage A checkout must be at ./clean, immutable SHA bed7fc80...
#[path = "../clean/src/lat.rs"]
mod lat;

use std::io::{self, BufRead, BufWriter, Write};

// Stage A FORMAT.md maps these permutation words to numeric codes.
// This is a transport adapter convention, NOT a clean-room code change.
const A_WORDS: [&str; 6] = ["012", "102", "210", "021", "120", "201"];

fn digit(word: &str) -> Result<u8, String> {
    A_WORDS.iter().position(|w| *w == word).map(|x| x as u8)
        .ok_or_else(|| format!("invalid 3-letter permutation: {word}"))
}
fn cfg_from_engine(line: &str) -> Result<(u64, lat::Cfg), String> {
    let fields: Vec<&str> = line.trim_end().split('|').collect();
    if fields.len() != 3 || fields[0] != "E" {
        return Err("expected E|key=...|links=...".into());
    }
    let key: u64 = fields[1].strip_prefix("key=").ok_or("missing engine key")?
        .parse().map_err(|_| "bad engine key")?;
    let links = fields[2].strip_prefix("links=").ok_or("missing links")?;
    let words: Vec<&str> = links.split(',').collect();
    if words.len() != 24 { return Err("expected 24 words".into()); }
    let mut cfg: lat::Cfg = [0; 24];
    for (slot, w) in words.iter().enumerate() {
        let (x, y, dir) = if slot < 12 {
            (slot / 4, slot % 4, 0)
        } else {
            ((slot - 12) / 4, (slot - 12) % 4, 1)
        };
        let aidx = 2 * (x + 3 * y) + dir;
        cfg[aidx] = digit(w)?;
    }
    Ok((key, cfg))
}
fn hword(ctx: &lat::Ctx, u: &lat::Cfg, x: usize, y: usize) -> &'static str {
    A_WORDS[ctx.hol(u, lat::site(x, y)) as usize]
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ctx = lat::Ctx::new(0);
    let stdin = io::stdin();
    let mut out = BufWriter::new(io::stdout().lock());
    let mut count = 0usize;
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() { continue; }
        let (engine_key, u) = cfg_from_engine(&line).map_err(io::Error::other)?;
        let (label, size, weight) = ctx.class_info(&u);
        let mag = ctx.mag_total(&u);
        let cfg: String = u.iter().map(|x| (b'0' + *x) as char).collect();
        writeln!(out,
            "B|engine={engine_key}|stage_key={label}|M={mag}|size={size}|W={}|h13={}|h23={}|h22={}|h03={}|cfg={cfg}|json=0",
            size * weight, hword(&ctx,&u,1,3), hword(&ctx,&u,2,3),
            hword(&ctx,&u,2,2), hword(&ctx,&u,0,3))?;
        count += 1;
    }
    out.flush()?;
    eprintln!("BULK_LABEL|PASS|records={count}|using_unchanged_stageA_lat_rs=1|coupling=none");
    Ok(())
}
