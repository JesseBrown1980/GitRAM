#!/usr/bin/env bash
# Frozen Stage B comparison wrapper: no r argument and no near-r9 calculation.
set -euo pipefail
m0="$1"
case "$m0" in
  11) key=e25b8b392161f6f02d933f6a1e78643b3ff95ae3fb69b72b02a36c2c8a5f3a7e; row=8369f4a981a888f4e028b155f3306371c2eacb421bed9bbc444b86e7ef862884 ;;
  12) key=942c2eed527aa2c327ce2aa5e811c0c64b0c524a46671f4ccb99ee1086ff5479; row=f835b2e1ded4b7f3f38d9d7f3f8437ebfa0a6ede53783f224ebbd1e9b4b7dee7 ;;
  13) key=b67d54ab32d0c33ac8b1cbd783fa6579c0f7423a342c169a58e4b7c8e86bbc94; row=902be763e0019b52bd393bffad6abff0d3cd8a9f4ddf8a6f93e8dd283a56654c ;;
  *) echo 'REFUSE: cutoff must be 11,12,13'; exit 1 ;;
esac
mkdir -p out/stage/replay out/engine
test "$(git -C cleanroom rev-parse HEAD)" = bed7fc80e5d21d4e4f388764507f818633c377ac
test "$(git -C cleanroom rev-parse HEAD^)" = b02c892636714ee03c3fac71642f6d9d754b83d0
test "$(git -C finite rev-parse HEAD)" = e74d8a46e86034096fd9c9231c87f594f17a93bf
src="$GITHUB_WORKSPACE/finite/independent/y131-floor-cert-freeze-20261110/work/design"
(cd "$src/../.." && grep -E '  work/design/(rs/src/main.rs|rs/Cargo.toml|rs/Cargo.lock|out/core_keys_m(11|12|13).bin)$' FREEZE-PIN.sha256 | sha256sum -c -) | tee out/pincheck.log
keys="$src/out/core_keys_m$m0.bin"
test "$(sha256sum "$keys" | cut -d' ' -f1)" = "$key"
stage="out/stage/rows-m$m0.hbp"
gzip -dc "cleanroom/results/main/rows-m$m0.hbp.gz" > "$stage"
expected="$(sed -n 's/^ROWS|file=rows-m[0-9]*.hbp|sha256=\([0-9a-f]*\)|json=0$/\1/p' "cleanroom/results/main/summary-m$m0.hbp")"
test -n "$expected"
test "$(sha256sum "$stage" | cut -d' ' -f1)" = "$expected"
echo "STAGE_A_FROZEN_ROWS|PASS|m0=$m0|sha256=$expected" | tee out/stage-input.log
sha256sum stageb/*.py stageb/lookup_stagea.rs cleanroom/PREDICT-STAGE-A.hbp cleanroom/FORMAT.md | tee out/source-manifest.sha256

# Compile untouched cleanroom and independently replay every final-build core.
(cd cleanroom && cargo +1.81.0 clippy --all-targets -- -D warnings) > out/cleanroom-clippy.log 2>&1
(cd cleanroom && CARGO_TARGET_DIR="$RUNNER_TEMP/clean-stageb" cargo +1.81.0 build --release --locked) > out/cleanroom-build.log 2>&1
cleanbin="$RUNNER_TEMP/clean-stageb/release/cleanrows"
sha256sum "$cleanbin" > out/cleanroom-binary.sha256
/usr/bin/time -v -o out/stage-replay.time "$cleanbin" run "$m0" "$GITHUB_WORKSPACE/out/stage/replay" "$GITHUB_WORKSPACE/cleanroom/PREDICT-STAGE-A.hbp" --threads 2 --rep sample > out/stage-replay.log
test "$(sha256sum "out/stage/replay/rows-m$m0.hbp" | cut -d' ' -f1)" = "$expected"
grep -q 'all_checks=PASS' out/stage-replay.log
echo "STAGE_A_FINAL_BINARY_REPLAY|PASS|m0=$m0" | tee -a out/stage-replay.log

# Publish three exact hand-built configurations to the real cleanroom lookup.
rustc +1.81.0 --edition 2021 -O stageb/lookup_stagea.rs -o out/lookup-stagea 2> out/lookup-build.log
for k in 1 3 25194240; do python3 -I -B stageb/engine_key_decoder.py --one "$k"; done > out/kat-engine.hbp
./out/lookup-stagea --known < out/kat-engine.hbp > out/kat-stagea.hbp
python3 - "$GITHUB_WORKSPACE/out/kat-stagea.hbp" "$cleanbin" "$GITHUB_WORKSPACE/cleanroom/label.py" <<'PY' | tee out/kat-verified.log
import subprocess,sys
expected={1:4,3:6,25194240:7}
for line in open(sys.argv[1]):
    d=dict(x.split("=",1) for x in line.strip().split("|")[1:-1])
    k=int(d["engine_key"])
    assert k in expected and int(d["M"])==expected[k]
    cfg=d["cfg"]
    assert len(cfg)==24 and all(x in "012345" for x in cfg)
    for cmd in ([sys.argv[2],"label",cfg],[sys.executable,"-I","-B",sys.argv[3],cfg]):
        output=subprocess.check_output(cmd,text=True)
        p=dict(x.split("=",1) for x in output.strip().split("|")[1:-1])
        for key,ref in (("M",d["M"]),("class_key",d["stage_key"]),("size",d["size"]),("W",d["W"])):
            assert p[key]==ref,(k,key,p[key],ref)
    print("STAGE_A_REAL_LOOKUP|PASS|engine_key=%d|M=%d|two_checks_agree=1"%(k,expected[k]))
PY

# Produce all engine configurations and map each through the genuine Stage A class lookup.
/usr/bin/time -v -o out/decoder.time python3 -I -B stageb/engine_key_decoder.py \
  --keys-file "$keys" --m0 "$m0" --pin-sha256 "$key" --output "$GITHUB_WORKSPACE/out/engine-transport.hbp" > out/decoder.log
/usr/bin/time -v -o out/lookup.time ./out/lookup-stagea < out/engine-transport.hbp > out/class-map.hbp
gzip -n -c out/class-map.hbp > out/class-map.hbp.gz

# Modify ONLY temporary checked-out v1 Rust source to add an instrument-only dump.
cp "$src/rs/src/main.rs" out/engine-original.rs
python3 -I -B stageb/patch_engine_rowdump.py "$src/rs/src/main.rs" > out/engine-patch.log
diff -u out/engine-original.rs "$src/rs/src/main.rs" > out/engine-patch.diff || test "$?" = 1
(cd "$src/rs" && CARGO_TARGET_DIR="$RUNNER_TEMP/eng-stageb" cargo +1.81.0 build --release --locked -j1) > out/engine-build.log 2>&1
engbin="$RUNNER_TEMP/eng-stageb/release/floorcert"
sha256sum "$engbin" > out/engine-instrumented-binary.sha256
cp "$keys" "out/engine/core_keys_m$m0.bin"
/usr/bin/time -v -o out/engine-dump.time "$engbin" stageb-dump --m0 "$m0" --threads 2 --gates 0 --images 0 --dir "$GITHUB_WORKSPACE/out/engine" > out/engine-dump.log
grep -Fq "rowhash=$row|keyshash=$key" out/engine-dump.log
fp="$(sed -n 's/^FINGERPRINT|m0=[0-9]*|rounds=3|sha256=\([0-9a-f]*\)|.*/\1/p' "cleanroom/results/main/summary-m$m0.hbp")"
test -n "$fp"
grep -Fq "STAGEB_FINGERPRINT|m0=$m0|sha256=$fp|" out/engine-dump.log
sha256sum "out/engine/stageb_engine_m$m0.hbp" > out/engine-rows.sha256
gzip -n -c "out/engine/stageb_engine_m$m0.hbp" > out/engine-rows.hbp.gz
gzip -n -c out/engine-transport.hbp > out/engine-transport.hbp.gz

# Full exact comparison, with an independently reported first counterexample on any mismatch.
/usr/bin/time -v -o out/comparison.time python3 -I -B stageb/stageb_compare.py \
  "$stage" "out/engine/stageb_engine_m$m0.hbp" out/class-map.hbp "$m0" | tee out/full-comparison.log
grep -Fq "STAGEB_FULL_ROWS|PASS|m0=$m0|" out/full-comparison.log

for logfile in out/stage-replay.time out/decoder.time out/lookup.time out/engine-dump.time out/comparison.time; do
  awk -F: '/Maximum resident set size/ {gsub(/ /, "", $2); v=$2; seen=1} END {if(!seen || v<1 || v>=2097152)exit 1; printf "RESOURCE|PASS|peak_kB=%d\n",v}' "$logfile" | tee -a out/resources.log
done
echo "STAGEB_FINAL|PASS|m0=$m0|all_rows=EXACT|coupling=none|r9_runs=0" | tee out/decision.hbp
