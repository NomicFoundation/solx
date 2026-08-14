#!/usr/bin/env python3
"""Generates a one-contract-per-file corpus for the pool-mode benchmark.

One contract per file because the slang frontend compiles only the first
contract of each file, so file count is the unit of parallelism. Each unit
of 20 files mixes stack-pressure, dispatch-heavy, switch-heavy, and
storage-churn contracts to spread work across the pipeline.

Usage: gen_corpus.py <output-dir> [multiplier]
"""
import os
import sys

outdir = sys.argv[1]
M = int(sys.argv[2]) if len(sys.argv) > 2 else 1
os.makedirs(outdir, exist_ok=True)


def emit(name, body):
    with open(os.path.join(outdir, f"{name}.sol"), "w") as f:
        f.write(body + "\n")


for m in range(M):
    sfx = f"_{m}" if M > 1 else ""

    # Stack pressure just below the spill threshold (~16 live locals of
    # this shape): stresses stackification without triggering the
    # overflow-retry path, which would skew timing.
    for i in range(8):
        n = 9 + i
        decls = "\n    ".join(f"uint x{j} = gasleft();" for j in range(n))
        total = " + ".join(f"x{j}" for j in reversed(range(n)))
        emit(f"Deep{i}{sfx}", f"""contract Deep{i}{sfx} {{
  function f() public view returns (uint) {{
    {decls}
    return {total};
  }}
}}""")

    for i in range(4):
        fns = "\n  ".join(
            f"function fn{i}_{j}(uint a) public pure returns (uint) {{ return a + {j}; }}"
            for j in range(40))
        emit(f"Dispatch{i}{sfx}", f"contract Dispatch{i}{sfx} {{\n  {fns}\n}}")

    for i, step in enumerate([1, 7]):
        cases = "\n    ".join(
            f"if (k == {j * step}) return {j * 3 + 1};" for j in range(64))
        emit(f"Switch{i}{sfx}", f"""contract Switch{i}{sfx} {{
  function pick(uint k) public pure returns (uint) {{
    {cases}
    return 0;
  }}
}}""")

    for i in range(6):
        emit(f"Churn{i}{sfx}", f"""contract Churn{i}{sfx} {{
  mapping(uint => uint) m;
  uint[] arr;
  string s;
  event E(uint indexed k, uint v);
  modifier guarded() {{ require(msg.sender != address(0), "zero"); _; }}
  function put(uint k, uint v) public guarded {{
    m[k] = v;
    arr.push(v + {i});
    emit E(k, v);
  }}
  function cat(string memory a) public {{
    s = string(abi.encodePacked(s, a));
  }}
  function sum(uint n) public view returns (uint acc) {{
    for (uint j = 0; j < n; ++j) acc += m[j] + arr.length;
  }}
}}""")

print(f"generated {20 * M} files in {outdir}")
