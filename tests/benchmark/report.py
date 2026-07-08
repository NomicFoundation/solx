#!/usr/bin/env python3
"""Render the compile-time benchmark report from run.sh's hyperfine output.

Reads every per-fixture hyperfine JSON in the given directory and writes a
markdown report with one table per fixture. The second benchmarked command is
the baseline (run.sh passes candidate first, baseline second): each row shows
its slowdown/speedup relative to it.
"""

import argparse
import json
import pathlib
import sys

REGRESSION_HIGHLIGHT = 1.05  # flag candidates >5% slower than baseline


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--dir", required=True, type=pathlib.Path,
                        help="run.sh --out directory")
    parser.add_argument("--out", required=True, type=pathlib.Path,
                        help="markdown report path")
    args = parser.parse_args()

    lines = ["## Compile-time benchmark (`--standard-json`, EVMLA pipeline)", ""]

    versions = args.dir / "versions.txt"
    if versions.exists():
        lines += ["```", versions.read_text().rstrip(), "```", ""]

    flagged = False
    for result_path in sorted(args.dir.glob("*.json")):
        results = json.loads(result_path.read_text())["results"]
        if len(results) > 1:
            baseline = results[1]["mean"]
        lines += [f"### {result_path.stem}", "",
                  "| binary | mean ± σ | min … max | vs baseline |",
                  "|---|---|---|---|"]
        for result in results:
            relative = ""
            if len(results) > 1 and baseline > 0:
                ratio = result["mean"] / baseline
                marker = " ⚠️" if ratio > REGRESSION_HIGHLIGHT and result is results[0] else ""
                relative = f"{ratio:.3f}×{marker}"
                flagged = flagged or bool(marker)
            lines.append(
                f"| {result['command']} "
                f"| {result['mean']:.3f} s ± {result['stddev']:.3f} s "
                f"| {result['min']:.3f} s … {result['max']:.3f} s "
                f"| {relative} |"
            )
        lines.append("")

    if flagged:
        lines += ["⚠️ = candidate more than "
                  f"{(REGRESSION_HIGHLIGHT - 1) * 100:.0f}% slower than baseline.", ""]

    args.out.write_text("\n".join(lines))
    print(f"wrote {args.out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
