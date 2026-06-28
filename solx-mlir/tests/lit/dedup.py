#!/usr/bin/env python3
"""
Deterministic deduplicator for the solx-mlir LIT suite, grounded in the dialect.

Run:  python3 solx-mlir/tests/lit/dedup.py [--apply] [--covset-dir DIR]

There are no heuristics and no judgment. Every input is derived from the
toolchain, so the result is reproducible:

  op-set(test)  the distinct sol.* / yul.* dialect OPERATIONS a test emits, taken
                from `solx --emit-mlir=sol` and intersected with the authoritative
                operation list parsed from the dialect .td files. The intersection
                drops types (!sol.array, !sol.ptr) and attributes (#sol<...>).
  covset(test)  the solx-slang/src + solx-mlir/src source lines the test covers,
                from an instrument-coverage build, executed through llvm-lit so the
                test's own RUN line (flags, evm-version, multi-contract) is honoured.

A duplicate is defined, not guessed:

  two tests are duplicates  <=>  identical op-set AND identical covset.

Dropping one member of such a cluster provably loses neither a dialect op nor a
source line. The tool keeps a deterministic canonical (lexicographically-first
file) and removes the rest. It is purely subtractive -- it never authors or merges
test content, so it cannot introduce slop.

Dialect/feature coherence is reported, not assumed:
  * op -> [tests] : which Solidity features (test files) exercise each op,
  * gaps          : dialect ops with no test at all,
  * near-dups     : tests with an identical op-set but DIFFERENT coverage. These
                    are surfaced for a human; the tool never removes them, because
                    removal would drop source lines and merging would be authoring.

Prerequisite for covset computation (skip with --covset-dir pointing at a cache of
`<test-basename>` files, one `file:line` per line):
  RUSTFLAGS="-C instrument-coverage" cargo build -p solx-slang -p solx-mlir -p solx \\
    --no-default-features --features slang --target-dir target-slang/cov
"""
import glob, hashlib, os, re, subprocess, sys

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))   # repo root
LIT  = HERE
BIN  = os.environ.get("SOLX_DEDUP_BIN", f"{ROOT}/target-slang/cov/debug/solx")
TOOLS = f"{ROOT}/target-llvm/target-final/bin"                  # fork llvm-lit + FileCheck
SRC  = [f"{ROOT}/solx-slang/src", f"{ROOT}/solx-mlir/src"]

OP_RE  = re.compile(r'(?:sol|yul)\.[a-z_0-9]+')
RUN_RE = re.compile(r'//\s*RUN:\s*(.*)')
DEF_RE = re.compile(r'\bdef\s+\w+\s*:\s*[^{};]*?"([a-z][a-z_0-9]*)"', re.S)
# .td tokens that are the dialect name / a type / a canonicalization rule, not an op
DENYLIST = {"sol.sol", "sol.bool", "sol.eq", "yul.yul", "yul.eq"}

def rustc(*args):
    return subprocess.run(["rustc", *args], capture_output=True, text=True).stdout.strip()

def prof_bin():
    sysroot = rustc("--print", "sysroot")
    host = next((l.split()[1] for l in rustc("-vV").splitlines() if l.startswith("host:")), "")
    return f"{sysroot}/lib/rustlib/{host}/bin"

def authoritative_ops():
    ops = set()
    for d, fn in (("Sol", "SolOps.td"), ("Yul", "YulOps.td")):
        txt = open(f"{ROOT}/solx-llvm/mlir/include/mlir/Dialect/{d}/{fn}").read()
        ops |= {f"{d.lower()}.{m.group(1)}" for m in DEF_RE.finditer(txt)}
    return ops - DENYLIST

def solx_cmd(path):
    """The test's own solx invocation (so flags like --evm-version are honoured)."""
    for line in open(path):
        m = RUN_RE.search(line)
        if not m or "solx" not in m.group(1):
            continue
        cmd = re.sub(r'^\s*not\s+', '', m.group(1).split("|")[0].split("2>")[0].strip())
        return [BIN] + cmd.replace("%s", path).split()[1:]
    return [BIN, "--emit-mlir=sol", path]

def op_set(path, ops):
    env = dict(os.environ, LLVM_PROFILE_FILE="/dev/null")
    try:
        out = subprocess.run(solx_cmd(path), capture_output=True, text=True, timeout=120, env=env).stdout
    except Exception:
        out = ""
    return frozenset(t for t in OP_RE.findall(out) if t in ops)

def covset_for(tests, covset_dir):
    """Per-test covered-source-line sets. Reuse a cache dir if given, else compute."""
    if covset_dir:
        return {t: _read_covset(os.path.join(covset_dir, t[:-4])) for t in tests}
    prof = prof_bin()
    work = subprocess.run(["mktemp", "-d"], capture_output=True, text=True).stdout.strip()
    out = {}
    for i, t in enumerate(tests, 1):
        pf = f"{work}/pf"; subprocess.run(["rm", "-rf", pf]); os.makedirs(pf)
        env = dict(os.environ, SOLX_LIT_TARGET="cov",
                   LLVM_PROFILE_FILE=f"{pf}/p-%p-%m.profraw",
                   PATH=f"{TOOLS}:{os.environ['PATH']}")
        subprocess.run([f"{TOOLS}/llvm-lit", "-q", f"{LIT}/{t}"],
                       env=env, capture_output=True)
        raw = glob.glob(f"{pf}/*.profraw")
        if not raw:
            out[t] = frozenset(); continue
        subprocess.run([f"{prof}/llvm-profdata", "merge", "-sparse", *raw, "-o", f"{pf}/pd"],
                       capture_output=True)
        lcov = subprocess.run([f"{prof}/llvm-cov", "export", "--format=lcov",
                               f"--instr-profile={pf}/pd", BIN, *SRC],
                              capture_output=True, text=True).stdout
        out[t] = _parse_lcov(lcov)
        print(f"\r  covset {i}/{len(tests)}", end="", file=sys.stderr)
    print("", file=sys.stderr)
    return out

def _read_covset(p):
    return frozenset(l.strip() for l in open(p)) if os.path.exists(p) else frozenset()

def _parse_lcov(lcov):
    s, cov = None, set()
    for line in lcov.splitlines():
        if line.startswith("SF:"): s = line[3:]
        elif line.startswith("DA:"):
            ln, cnt = line[3:].split(",")[:2]
            if int(cnt) > 0: cov.add(f"{s}:{ln}")
    return frozenset(cov)

def main():
    apply = "--apply" in sys.argv
    covset_dir = next((a.split("=", 1)[1] for a in sys.argv if a.startswith("--covset-dir=")), None)
    if "--covset-dir" in sys.argv:
        covset_dir = sys.argv[sys.argv.index("--covset-dir") + 1]

    ops = authoritative_ops()
    tests = sorted(os.path.basename(p) for p in glob.glob(f"{LIT}/*.sol"))
    print(f"scanning {len(tests)} tests against {len(ops)} dialect ops", file=sys.stderr)
    opset = {t: op_set(f"{LIT}/{t}", ops) for t in tests}
    cov = covset_for(tests, covset_dir)

    clusters = {}
    for t in tests:
        key = hashlib.md5((repr(sorted(opset[t])) + "|" + repr(sorted(cov[t]))).encode()).hexdigest()
        clusters.setdefault(key, []).append(t)
    dups = sorted([sorted(v) for v in clusters.values() if len(v) > 1])
    removable = [d for c in dups for d in c[1:]]

    print("=== EXACT DUPLICATES (identical op-set AND covset) ===")
    print("  none" if not dups else "", end="")
    for c in dups:
        print(f"  keep {c[0]}")
        for d in c[1:]:
            print(f"    drop {d}")

    by_op = {}
    for t in tests:
        if opset[t]:
            by_op.setdefault(opset[t], []).append(t)
    near = sorted(sorted(g) for g in by_op.values() if len(g) > 1 and len({cov[t] for t in g}) > 1)
    print(f"\n=== NEAR-DUPLICATES (same op-set, different coverage) [reported only] ===")
    print("  none" if not near else "", end="")
    for g in near:
        print(f"  {', '.join(g)}")

    op_tests = {o: [t for t in tests if o in opset[t]] for o in ops}
    gaps = sorted(o for o in ops if o.startswith("sol.") and not op_tests[o])
    print(f"\n=== DIALECT COHERENCE ===")
    print(f"  ops: {len(ops)} (sol={sum(o.startswith('sol.') for o in ops)}, yul={sum(o.startswith('yul.') for o in ops)})")
    print(f"  sol ops with no test: {len(gaps)}  {gaps}")

    print(f"\n=== SUMMARY ===")
    print(f"  tests {len(tests)} -> {len(tests) - len(removable)}   exact-dup clusters {len(dups)}   near-dup clusters {len(near)}")
    if apply and removable:
        for d in removable:
            os.remove(f"{LIT}/{d}")
        print(f"  APPLIED: removed {len(removable)}: {', '.join(removable)}")
    elif removable:
        print(f"  dry-run: pass --apply to remove {len(removable)} non-canonical duplicates")

if __name__ == "__main__":
    main()
