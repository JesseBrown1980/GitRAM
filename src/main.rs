//! Stage A clean-room row reconstruction: S3 gauge orbits on the periodic 3x4 lattice.
//! Exact integers only.
//!
//!   cleanrows run <m0> <outdir> <predict.hbp> [--mut N] [--rep all|sample|none] [--orbit-balance]
//!                 [--threads N] [--tag NAME]
//!   cleanrows label <24 digits 0..5 in link order> [rows file]

mod checks;
mod enumr;
mod lat;
mod rows;
mod sha;

use std::io::Write;
use std::time::Instant;

fn vm_hwm_kib() -> u64 {
    let s = std::fs::read_to_string("/proc/self/status").unwrap_or_default();
    for line in s.lines() {
        if let Some(rest) = line.strip_prefix("VmHWM:") {
            return rest.split_whitespace().next().and_then(|x| x.parse().ok()).unwrap_or(0);
        }
    }
    0
}

fn parse_cfg(s: &str) -> lat::Cfg {
    let d: Vec<u8> = s.bytes().filter(|b| b.is_ascii_digit()).map(|b| b - b'0').collect();
    assert!(d.len() == lat::NL && d.iter().all(|x| *x < 6), "need 24 digits in 0..5");
    let mut u = [0u8; lat::NL];
    u.copy_from_slice(&d);
    u
}

fn label(args: &[String]) {
    let ctx = lat::Ctx::new(0);
    let u = parse_cfg(&args[0]);
    let m = ctx.mag_total(&u);
    let (key, size, w) = ctx.class_info(&u);
    let mut id = "not_looked_up".to_string();
    if args.len() > 1 {
        id = "absent".to_string();
        let text = std::fs::read_to_string(&args[1]).expect("rows file");
        let want = format!("key={}", key);
        for line in text.lines() {
            let f: Vec<&str> = line.split('|').collect();
            if f[0] == "R" && f[2] == want {
                id = f[1].trim_start_matches("id=").to_string();
            }
        }
    }
    let core = |m0: u32| if m > 0 && m <= m0 { 1 } else { 0 };
    println!(
        "LABEL|class_key={}|M={}|size={}|w={}|W={}|flat={}|core_m11={}|core_m12={}|core_m13={}|id={}|json=0",
        key,
        m,
        size,
        w,
        size * w,
        if m == 0 { 1 } else { 0 },
        core(11),
        core(12),
        core(13),
        id
    );
}

fn run(args: &[String]) {
    let t0 = Instant::now();
    let m0: u32 = args[0].parse().expect("m0");
    let outdir = args[1].clone();
    let pred = checks::read_pred(&args[2]);
    let mut mutation = 0u8;
    let mut rep = "sample".to_string();
    let mut orbit_balance = false;
    let mut threads = 8usize;
    let mut tag = String::new();
    let mut i = 3;
    while i < args.len() {
        match args[i].as_str() {
            "--mut" => {
                mutation = args[i + 1].parse().expect("mutation");
                i += 1;
            }
            "--rep" => {
                rep = args[i + 1].clone();
                i += 1;
            }
            "--threads" => {
                threads = args[i + 1].parse().expect("threads");
                i += 1;
            }
            "--tag" => {
                tag = args[i + 1].clone();
                i += 1;
            }
            "--orbit-balance" => orbit_balance = true,
            x => panic!("unknown argument {}", x),
        }
        i += 1;
    }
    let ctx = lat::Ctx::new(mutation);
    let mut out: Vec<String> = Vec::new();
    let mut chk: Vec<(String, bool, String)> = Vec::new();
    out.push(format!("SUMMARY-HEAD|stage=A|m0={}|mutation={}|threads={}|rep_mode={}|json=0", m0, mutation, threads, rep));

    let e = enumr::enumerate(&ctx, m0, threads, orbit_balance);
    let t_enum = t0.elapsed().as_millis();
    let (p, d) = checks::c1_prediction(&pred, &e, m0);
    chk.push(("C1_enumeration_equals_independent_count".into(), p, d));
    out.push(format!(
        "FLAT|M=0|orbits={}|weight={}|pred_orbits={}|pred_weight={}|json=0",
        e.orbits[0], e.weight[0], pred.orbits[0], pred.weight[0]
    ));
    let (mut cc, mut co, mut cw) = (0u64, 0u64, 0u64);
    for m in 1..=(m0 as usize) {
        cc += e.classes[m];
        co += e.orbits[m];
        cw += e.weight[m];
        out.push(format!(
            "PERM|M={}|classes={}|orbits={}|weight={}|orbits_from_class_sizes={}|weight_from_classes={}|pred_orbits={}|pred_weight={}|match={}|cum_classes={}|cum_orbits={}|cum_weight={}|json=0",
            m,
            e.classes[m],
            e.orbits[m],
            e.weight[m],
            e.class_orbits[m],
            e.class_weight[m],
            pred.orbits[m],
            pred.weight[m],
            if e.orbits[m] == pred.orbits[m] && e.weight[m] == pred.weight[m] && e.class_orbits[m] == e.orbits[m] && e.class_weight[m] == e.weight[m] { "YES" } else { "NO" },
            cc,
            co,
            cw
        ));
    }
    let mut hist = [0u64; 49];
    for r in e.reps.iter() {
        hist[r.size as usize] += 1;
    }
    for (s, c) in hist.iter().enumerate() {
        if *c > 0 {
            out.push(format!("HIST|class_size={}|classes={}|json=0", s, c));
        }
    }
    out.push(format!("ENUM|leaves={}|classes={}|millis={}|json=0", e.leaves, e.reps.len(), t_enum));

    if orbit_balance {
        let (p, d) = checks::c5_orbit_balance(&ctx, &e.all, m0, threads);
        chk.push(("C5_orbit_level_detailed_balance".into(), p, d));
    }
    drop(e.all);

    let mut table = rows::build(&ctx, m0, e.reps, threads);
    let t_rows = t0.elapsed().as_millis();
    // row-table mutations (controls 4, 5, 6)
    let n = table.rows.len();
    if mutation == 4 {
        table.reps[n / 2].size += 1;
        out.push(format!("MUTATION|kind=class_size|class={}|json=0", n / 2));
    }
    if mutation == 5 {
        let mut a = n / 2;
        loop {
            let pos = table.rows[a].core.iter().position(|e| e.0 != a as u32);
            if let (Some(k), false) = (pos, table.rows[a].tail.is_empty()) {
                if table.rows[a].core[k].1 == 1 {
                    table.rows[a].core.remove(k);
                } else {
                    table.rows[a].core[k].1 -= 1;
                }
                table.rows[a].tail[0].1 += 1;
                break;
            }
            a = (a + 1) % n;
        }
        out.push(format!("MUTATION|kind=move_core_to_tail|class={}|json=0", a));
    }
    if mutation == 6 {
        let mut t = table.tails.len() / 2;
        while table.tails[t].len() < 2 {
            t = (t + 1) % table.tails.len();
        }
        let old = table.tails[t][1];
        table.tails[t][1] = (old + 1) % n as u32;
        table.tails[t][1..].sort_unstable();
        out.push(format!("MUTATION|kind=back_multiset|tailset={}|json=0", t));
    }
    chk.push(("C9_core_closure_no_unknown_class".into(), table.misses == 0, format!("unknown_targets={}", table.misses)));
    let (p, d) = checks::c2_class_sums(&pred, &table);
    chk.push(("C2_class_sizes_sum_to_independent_count".into(), p, d));
    let (p, d) = checks::c8_structure(&table);
    chk.push(("C8_row_structure".into(), p, d));
    let (p, d) = checks::c6_class_balance(&table);
    chk.push(("C6_class_level_detailed_balance".into(), p, d));
    let (p, d) = checks::c7_two_step(&table, threads);
    chk.push(("C7_two_step_tail_balance".into(), p, d));
    let t_bal = t0.elapsed().as_millis();
    if rep != "none" {
        let (p, d) = checks::c4_rep(&ctx, &table, rep == "all", 101, 50, threads);
        chk.push((format!("C4_representative_independence_{}", rep), p, d));
    }
    let t_rep = t0.elapsed().as_millis();

    let (nc, nz, flat, te, back) = table.totals();
    out.push(format!(
        "TOTALS|m0={}|core_classes={}|core_core_nonzero={}|flat_count_sum={}|tail_entries={}|back_multiset_total={}|tail_sets_distinct={}|json=0",
        m0,
        nc,
        nz,
        flat,
        te,
        back,
        table.tails.len()
    ));
    let name = format!("{}/rows-m{}{}.hbp", outdir, m0, tag);
    let sha = table.write(&name).expect("write rows");
    out.push(format!("ROWS|file=rows-m{}{}.hbp|sha256={}|json=0", m0, tag, sha));
    if table.misses == 0 && mutation != 6 {
        let (fp, distinct) = table.fingerprint(threads);
        out.push(format!("FINGERPRINT|m0={}|rounds=3|sha256={}|distinct_colour3={}|classes={}|json=0", m0, fp, distinct, nc));
    } else {
        out.push("FINGERPRINT|status=NOT_COMPUTED|reason=table_not_consistent|json=0".to_string());
    }
    let mut allpass = true;
    for (name, p, d) in chk.iter() {
        allpass &= *p;
        out.push(format!("CHECK|name={}|result={}|detail={}|json=0", name, if *p { "PASS" } else { "FAIL" }, d));
    }
    out.push(format!(
        "RUN|m0={}|mutation={}|all_checks={}|millis_enum={}|millis_rows={}|millis_balance={}|millis_rep={}|millis_total={}|vmhwm_kib={}|json=0",
        m0,
        mutation,
        if allpass { "PASS" } else { "FAIL" },
        t_enum,
        t_rows - t_enum,
        t_bal - t_rows,
        t_rep - t_bal,
        t0.elapsed().as_millis(),
        vm_hwm_kib()
    ));
    let text = out.join("\n") + "\n";
    let sname = format!("{}/summary-m{}{}.hbp", outdir, m0, tag);
    std::fs::File::create(&sname).expect("summary").write_all(text.as_bytes()).expect("summary write");
    print!("{}", text);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("run") => run(&args[2..]),
        Some("label") => label(&args[2..]),
        _ => eprintln!("usage: cleanrows run <m0> <outdir> <predict.hbp> [options] | cleanrows label <cfg> [rows file]"),
    }
}
