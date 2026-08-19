#!/usr/bin/env python3
"""Generates the pool-mode benchmark corpora.

Five corpora as subdirectories of the output dir, isolating the job-cost
x stack-too-deep matrix (one contract per file - the slang frontend
compiles only the first contract of a file, so file count is the unit
of parallelism):

- tiny-clean:    minimal contracts; measures pure per-job dispatch
                 overhead (serde, pipes, worker checkout).
- tiny-overflow: minimal contracts that hit stack-too-deep; the retry
                 costs one extra in-process compile on threads, but a
                 worker retire + respawn on the subprocess pool.
- big-clean:     ~3s-of-codegen contracts (40 functions x 64-way
                 chains); measures long-lived jobs.
- big-overflow:  the same big body plus overflowing functions - the
                 whole module recompiles on retry, so this is the
                 most expensive divergence point between the modes.
- mixed:         all four classes interleaved round-robin, the
                 realistic blend.

Usage: gen_corpus.py <output-dir> [scale]
"""
import os
import sys

outdir = sys.argv[1]
scale = float(sys.argv[2]) if len(sys.argv) > 2 else 1.0


def tiny_clean(name, i):
    return f"""contract {name} {{
  function f(uint a) public pure returns (uint) {{
    return a * {i + 3} + {i};
  }}
}}"""


def overflow_fn(fname, n):
    # Forward-order consumption of n live gasleft() values cannot be
    # scheduled on the EVM stack; n >= 48 reliably diagnoses overflow
    # and takes the module-flag retry path.
    decls = "\n    ".join(f"uint x{j} = gasleft();" for j in range(n))
    total = " + ".join(f"x{j}" for j in range(n))
    return f"""  function {fname}() public view returns (uint) {{
    {decls}
    return {total};
  }}"""


def tiny_overflow(name, i):
    return f"contract {name} {{\n{overflow_fn('f', 48 + i % 8)}\n}}"


def big_fns(count, salt):
    fns = []
    for f in range(count):
        cases = "\n    ".join(
            f"if (k == {j * 3 + f + salt}) return k * {j + 1} + {f};"
            for j in range(64))
        fns.append(f"""  function w{f}(uint k) public pure returns (uint) {{
    {cases}
    return k + {f};
  }}""")
    return "\n".join(fns)


def big_clean(name, i):
    return f"contract {name} {{\n{big_fns(40, i)}\n}}"


def big_overflow(name, i):
    return (f"contract {name} {{\n{big_fns(40, i)}\n"
            f"{overflow_fn('o0', 48 + i % 8)}\n"
            f"{overflow_fn('o1', 52 + i % 4)}\n}}")


CLASSES = {
    "tiny-clean": (tiny_clean, 200),
    "tiny-overflow": (tiny_overflow, 100),
    "big-clean": (big_clean, 12),
    "big-overflow": (big_overflow, 6),
}


def emit(subdir, fname, text):
    d = os.path.join(outdir, subdir)
    os.makedirs(d, exist_ok=True)
    with open(os.path.join(d, f"{fname}.sol"), "w") as f:
        f.write(text + "\n")


for sub, (gen, count) in CLASSES.items():
    n = max(1, int(count * scale))
    cname = sub.title().replace("-", "")
    for i in range(n):
        emit(sub, f"{cname}{i}", gen(f"{cname}{i}", i))

# Mixed: half-scale of each class, round-robin interleaved by filename
# so the pool schedules big/tiny/overflow jobs together.
queues = [
    (sub, gen, list(range(max(1, int(count * scale) // 2))))
    for sub, (gen, count) in CLASSES.items()
]
seq = 0
while any(q for _, _, q in queues):
    for sub, gen, q in queues:
        if not q:
            continue
        i = q.pop(0)
        cname = "Mx" + sub.title().replace("-", "")
        emit("mixed", f"m{seq:04d}_{cname}{i}", gen(f"{cname}{i}", i))
        seq += 1

print(f"generated corpora under {outdir}: "
      + ", ".join(f"{s}={max(1, int(c * scale))}" for s, (_, c) in CLASSES.items())
      + f", mixed={sum(max(1, int(c * scale)) // 2 for _, c in CLASSES.values())}")
