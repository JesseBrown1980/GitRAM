//! Independent Host8/3D-S3 streaming experiment. Rust 1.81, integers, ternary, zero JSON/Node/V8.
//! The 300 GB stream is the deterministic Host8 source generator, NOT the user's physical drive.
//! 20 disjoint workers; one final ordered HBP/HBI/SHA/SH/HASH commitment.
#![forbid(unsafe_code)]
#![deny(clippy::float_arithmetic, clippy::float_cmp, clippy::cast_precision_loss)]
mod sha256;
use sha256::hash_bytes;
use std::collections::{BTreeMap, BTreeSet};
use std::env;
use std::fs;

const TOTAL: u64 = 37_500_000_000;
const WINDOW: u64 = 13_824;
const SHARDS: u64 = 20;
const H27: i128 = 3_812_798_742_493;
const SEED: u64 = 0x518b_bb47_2a9d_b786;
const M1: u64 = 0xbf58_476d_1ce4_e5b9;
const M2: u64 = 0x94d0_49bb_1331_11eb;
const PERMS: [[usize; 3]; 6] = [
    [0, 1, 2], [0, 2, 1], [1, 0, 2], [1, 2, 0], [2, 0, 1], [2, 1, 0]
];
const TRANS: [u8; 3] = [1, 2, 5];

fn mix(mut z: u64) -> u64 {
    z = (z ^ (z >> 30)).wrapping_mul(M1);
    z = (z ^ (z >> 27)).wrapping_mul(M2);
    z ^ (z >> 31)
}
fn room_word(n: u64) -> u64 {
    assert!(n < TOTAL, "Host8 address outside canonical 300GB plane");
    let signed = (i128::from(n) - i128::from(TOTAL) / 2 + H27) as u64;
    mix(signed ^ SEED)
}
fn mul(a: u8, b: u8) -> u8 {
    let x = PERMS[usize::from(a)];
    let y = PERMS[usize::from(b)];
    let v = [x[y[0]], x[y[1]], x[y[2]]];
    PERMS.iter().position(|p| *p == v).expect("S3 closed") as u8
}
fn inv(a: u8) -> u8 {
    (0..6).find(|b| mul(a, *b) == 0).expect("inverse")
}
fn conjugate(g: u8, a: u8) -> u8 {
    mul(mul(g, a), inv(g))
}
fn flat_sectors() -> Vec<[u8; 3]> {
    let mut classes = BTreeSet::new();
    for a in 0..6 {
        for b in 0..6 {
            for c in 0..6 {
                if mul(a, b) != mul(b, a) || mul(a, c) != mul(c, a) || mul(b, c) != mul(c, b) {
                    continue;
                }
                let canon = (0..6).map(|g| [conjugate(g, a), conjugate(g, b), conjugate(g, c)])
                    .min().expect("six conjugations");
                classes.insert(canon);
            }
        }
    }
    assert_eq!(classes.len(), 21);
    classes.into_iter().collect()
}
fn site(x: usize, y: usize, z: usize) -> usize {
    x * 4 + y * 2 + z
}
fn shifted(x: usize, y: usize, z: usize, d: usize) -> (usize, usize, usize) {
    match d {
        0 => ((x + 1) % 2, y, z),
        1 => (x, (y + 1) % 2, z),
        2 => (x, y, (z + 1) % 2),
        _ => panic!("bad direction"),
    }
}
fn links(a: u8, b: u8, c: u8) -> [u8; 24] {
    let mut out = [0u8; 24];
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                let p = site(x, y, z) * 3;
                out[p] = if x == 1 { a } else { 0 };
                out[p + 1] = if y == 1 { b } else { 0 };
                out[p + 2] = if z == 1 { c } else { 0 };
            }
        }
    }
    out
}
fn magnetic(u: &[u8; 24]) -> u32 {
    let mut energy = 0;
    for x in 0..2 {
        for y in 0..2 {
            for z in 0..2 {
                for d in 0..3 {
                    for e in (d + 1)..3 {
                        let (dx, dy, dz) = shifted(x, y, z, d);
                        let (ex, ey, ez) = shifted(x, y, z, e);
                        let h = mul(
                            mul(
                                mul(u[site(x, y, z) * 3 + d], u[site(dx, dy, dz) * 3 + e]),
                                inv(u[site(ex, ey, ez) * 3 + d])
                            ),
                            inv(u[site(x, y, z) * 3 + e])
                        );
                        energy += match h {
                            0 => 0,
                            1 | 2 | 5 => 2,
                            3 | 4 => 3,
                            _ => panic!("S3 value"),
                        };
                    }
                }
            }
        }
    }
    energy
}
fn packed_rooms(u: &[u8; 24]) -> [u64; 2] {
    let mut trits = [0u8; 54];
    for (i, a) in u.iter().copied().enumerate() {
        trits[2 * i] = a % 3;
        trits[2 * i + 1] = a / 3;
    }
    let mut rooms = [0u64; 2];
    for i in 0..54 {
        rooms[i / 27] = rooms[i / 27] * 3 + u64::from(trits[i]);
    }
    rooms
}
fn physics(shard: u64) -> (u64, u64, u64, String) {
    let mut records = String::new();
    let mut sectors = 0;
    let mut singles = 0;
    let mut doubles = 0;
    for (i, triple) in flat_sectors().iter().enumerate() {
        if i as u64 % SHARDS != shard {
            continue;
        }
        let u = links(triple[0], triple[1], triple[2]);
        assert_eq!(magnetic(&u), 0, "flat group sector");
        let words = packed_rooms(&u);
        let mut one = BTreeMap::<u32, u64>::new();
        let mut two = BTreeMap::<u32, u64>::new();
        for k in 0..24 {
            for t in TRANS {
                let mut v = u;
                v[k] = mul(t, v[k]);
                *one.entry(magnetic(&v)).or_default() += 1;
                singles += 1;
            }
            for j in (k + 1)..24 {
                for t in TRANS {
                    for s in TRANS {
                        let mut v = u;
                        v[k] = mul(t, v[k]);
                        v[j] = mul(s, v[j]);
                        *two.entry(magnetic(&v)).or_default() += 1;
                        doubles += 1;
                    }
                }
            }
        }
        let row = format!("SECTOR|index={i}|a={}|b={}|c={}|host8room0={}|host8room1={}|one={one:?}|two={two:?}|json=0\n",
            triple[0], triple[1], triple[2], words[0], words[1]);
        records.push_str(&row);
        sectors += 1;
    }
    (sectors, singles, doubles, hash_bytes(records.as_bytes()))
}
fn stream(shard: u64) -> (u64, u64, u64, String) {
    let windows = TOTAL.div_ceil(WINDOW);
    let first = windows * shard / SHARDS;
    let last = windows * (shard + 1) / SHARDS;
    let mut chain = hash_bytes(b"HOST8_STREAM_V1|37_500_000_000|13_824|json=0");
    let mut count = 0;
    let mut bytes = 0;
    let mut buffer = Vec::with_capacity(WINDOW as usize * 8);
    for w in first..last {
        let start = w * WINDOW;
        let end = ((w + 1) * WINDOW).min(TOTAL);
        buffer.clear();
        for n in start..end {
            buffer.extend_from_slice(&room_word(n).to_le_bytes());
        }
        assert_eq!(buffer.len() as u64, (end - start) * 8);
        let sha = hash_bytes(&buffer);
        chain = hash_bytes(format!("{chain}|{w}|{}|{sha}", end - start).as_bytes());
        count += end - start;
        bytes += buffer.len() as u64;
    }
    assert_eq!(bytes, count * 8);
    (first, last, count, chain)
}
fn field<'a>(line: &'a str, key: &str) -> &'a str {
    line.split('|').skip(1).find_map(|kv| {
        let (k, v) = kv.split_once('=')?;
        (k == key).then_some(v)
    }).expect("missing record field")
}
fn num(line: &str, key: &str) -> u64 {
    field(line, key).parse().expect("bad integer field")
}
fn check_digest(s: &str) {
    assert_eq!(s.len(), 64, "SHA-256 hex length");
    assert!(s.bytes().all(|x| x.is_ascii_digit() || (b'a'..=b'f').contains(&x)));
}
fn run_shard(shard: u64) {
    assert!(shard < SHARDS);
    // Reproduce the published synthetic Host8 Sector 000 Window 000 fixture.
    let mut fixture = Vec::with_capacity(WINDOW as usize * 8);
    for n in 0..WINDOW {
        fixture.extend_from_slice(&room_word(n).to_le_bytes());
    }
    assert_eq!(hash_bytes(&fixture), "2dfe7d757ecb012475806b86fcdefc602680d042db836ffbcb2bfe924f2a18ee");
    let (first, last, rooms, room_sha) = stream(shard);
    let (sectors, singles, doubles, physics_sha) = physics(shard);
    let sh = &room_sha[..16];
    println!("HOST8_SHARD|id={shard}|first={first}|last={last}|rooms={rooms}|bytes={}|sectors={sectors}|singles={singles}|doubles={doubles}|sha={room_sha}|sh={sh}|physics_sha={physics_sha}|json=0", rooms * 8);
}
fn run_fanin(path: &str) {
    let hbp = fs::read_to_string(path).expect("read shard records");
    let lines: Vec<&str> = hbp.lines().collect();
    assert_eq!(lines.len(), SHARDS as usize);
    let windows = TOTAL.div_ceil(WINDOW);
    let mut room_total = 0;
    let mut sector_total = 0;
    let mut one_total = 0;
    let mut two_total = 0;
    for (i, line) in lines.iter().enumerate() {
        assert!(line.starts_with("HOST8_SHARD|") && line.ends_with("|json=0"));
        assert_eq!(num(line, "id"), i as u64);
        assert_eq!(num(line, "first"), windows * i as u64 / SHARDS);
        assert_eq!(num(line, "last"), windows * (i as u64 + 1) / SHARDS);
        let first = num(line, "first") * WINDOW;
        let last = (num(line, "last") * WINDOW).min(TOTAL);
        assert_eq!(num(line, "rooms"), last - first);
        assert_eq!(num(line, "bytes"), (last - first) * 8);
        check_digest(field(line, "sha"));
        check_digest(field(line, "physics_sha"));
        assert_eq!(field(line, "sh"), &field(line, "sha")[..16]);
        room_total += num(line, "rooms");
        sector_total += num(line, "sectors");
        one_total += num(line, "singles");
        two_total += num(line, "doubles");
    }
    assert_eq!(room_total, TOTAL);
    assert_eq!(sector_total, 21);
    assert_eq!(one_total, 21 * 24 * 3);
    assert_eq!(two_total, 21 * 276 * 9);
    let root = hash_bytes(hbp.as_bytes());
    let sh = &root[..16];
    println!("HBP|version=1|model=S3_2x2x2|transport=Host8_synthetic_generator|workers=20|rooms={room_total}|bytes={}|flat_sectors={sector_total}|one_link_checks={one_total}|two_link_checks={two_total}|sha={root}|sh={sh}|hash={}|json=0",
        room_total * 8, hash_bytes(format!("HOST8_GLOBAL|{root}|{sh}|{sector_total}").as_bytes()));
    println!("HBI|records=20|ordered=YES|complete=YES|source_sha256={root}|sh={sh}|json=0");
    println!("VERDICT|HOST8_3D_20|PASS|scope=full_synthetic_address_stream_and_flat_sector_transitions|spectral_gap=NOT_COMPUTED|r9_runs=0|json=0");
}
fn main() {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("shard") => {
            let id = args.get(2).expect("shard index").parse::<u64>().expect("integer index");
            run_shard(id);
        }
        Some("fan-in") => run_fanin(args.get(2).expect("records file")),
        _ => panic!("expected shard <0..19> or fan-in <records.hbp>"),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn count_and_s3_algebra() {
        assert_eq!(flat_sectors().len(), 21);
        for a in 0..6 {
            assert_eq!(mul(a, inv(a)), 0);
            assert_eq!(mul(inv(a), a), 0);
        }
        assert_eq!(TOTAL * 8, 300_000_000_000);
    }
    #[test]
    fn all_flat_and_packed() {
        for triple in flat_sectors() {
            let l = links(triple[0], triple[1], triple[2]);
            assert_eq!(magnetic(&l), 0);
            assert!(packed_rooms(&l).iter().all(|w| *w < 7_625_597_484_987));
        }
    }
}