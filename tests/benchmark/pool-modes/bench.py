#!/usr/bin/env python3
"""A/B wall-clock benchmark: in-process threads vs SOLX_SUBPROCESS workers.

Runs the two modes in interleaved pairs so turbo/thermal drift cancels -
single-shot timings mislead (the first heavy run rides boost clocks).
Pair 0 is warmup and excluded from the medians.

Contracts are passed as basenames with cwd at the corpus dir to stay
under the Windows command-line length limit.

Usage: bench.py --solx <binary> --corpus <dir> [--threads N] [--pairs N]
"""
import argparse
import os
import statistics
import subprocess
import sys
import time

parser = argparse.ArgumentParser()
parser.add_argument("--solx", required=True)
parser.add_argument("--corpus", required=True)
parser.add_argument("--threads", type=int, default=os.cpu_count())
parser.add_argument("--pairs", type=int, default=5)
parser.add_argument("--label", default=None)
args = parser.parse_args()
label = args.label or os.path.basename(os.path.normpath(args.corpus))

solx = os.path.abspath(args.solx)
files = sorted(f for f in os.listdir(args.corpus) if f.endswith(".sol"))
if not files:
    sys.exit(f"no .sol files in {args.corpus}")
cmd = [solx, "--threads", str(args.threads), "--bin", *files]


def run(subprocess_mode):
    env = dict(os.environ)
    env.pop("SOLX_SUBPROCESS", None)
    if subprocess_mode:
        env["SOLX_SUBPROCESS"] = "1"
    t0 = time.monotonic()
    r = subprocess.run(cmd, cwd=args.corpus, env=env,
                       stdout=subprocess.PIPE, stderr=subprocess.PIPE)
    wall = time.monotonic() - t0
    binaries = r.stdout.decode(errors="replace").count("\nBinary:")
    if r.returncode != 0 or binaries != len(files):
        sys.exit(f"mode={'subprocess' if subprocess_mode else 'threads'} "
                 f"rc={r.returncode} binaries={binaries}/{len(files)}\n"
                 f"{r.stderr.decode(errors='replace')[-2000:]}")
    return wall


threads, workers = [], []
for i in range(args.pairs):
    a = run(subprocess_mode=False)
    b = run(subprocess_mode=True)
    print(f"[{label}] pair {i}{' (warmup)' if i == 0 else ''}: "
          f"threads={a:.2f}s subprocess={b:.2f}s", flush=True)
    if i > 0:
        threads.append(a)
        workers.append(b)

mt, ms = statistics.median(threads), statistics.median(workers)
print(f"RESULT label={label} files={len(files)} threads_n={args.threads} "
      f"threads_median={mt:.2f}s subprocess_median={ms:.2f}s "
      f"ratio={ms / mt:.2f}")
