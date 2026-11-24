#!/usr/bin/env python3
"""Render the compile-time benchmark report from run.sh's hyperfine output.

Reads every per-scenario hyperfine JSON in the given directory and writes a
markdown report: one summary table with a row per scenario, then the full
hyperfine statistics in a collapsed section. The second benchmarked command is
the baseline (run.sh passes candidate first, baseline second): every other
binary gets a column with its slowdown/speedup relative to it, by median so
that one slow run does not move the ratio.
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

    scenarios = {
        path.stem: json.loads(path.read_text())["results"]
        for path in sorted(args.dir.glob("*.json"))
    }
    binaries = [result["command"] for result in next(iter(scenarios.values()), [])]
    baseline = binaries[1] if len(binaries) > 1 else None
    compared = [binary for binary in binaries if binary != baseline]

    lines = ["## Compile-time benchmark (`--standard-json`)", ""]

    versions = args.dir / "versions.txt"
    if versions.exists():
        lines += ["```", versions.read_text().rstrip(), "```", ""]

    header = ["scenario", *(f"{binary} median" for binary in binaries)]
    if baseline is not None:
        header += [f"{binary} vs {baseline}" for binary in compared]
    lines += ["| " + " | ".join(header) + " |",
              "|" + "---|" * len(header)]

    flagged = False
    for scenario, results in scenarios.items():
        medians = {result["command"]: result["median"] for result in results}
        row = [scenario, *(f"{medians[binary]:.3f} s" for binary in binaries)]
        if baseline is not None:
            for binary in compared:
                ratio = medians[binary] / medians[baseline]
                marker = (" ⚠️" if ratio > REGRESSION_HIGHLIGHT
                          and binary == binaries[0] else "")
                flagged = flagged or bool(marker)
                row.append(f"{ratio:.3f}×{marker}")
        lines.append("| " + " | ".join(row) + " |")
    lines.append("")

    if flagged:
        lines += [f"⚠️ = {binaries[0]} more than "
                  f"{(REGRESSION_HIGHLIGHT - 1) * 100:.0f}% slower than {baseline}.", ""]

    lines += ["<details><summary>Full statistics</summary>", "",
              "| scenario | binary | runs | median | mean ± σ | min … max |",
              "|---|---|---|---|---|---|"]
    for scenario, results in scenarios.items():
        for result in results:
            lines.append(
                f"| {scenario} | {result['command']} "
                f"| {len(result['times'])} "
                f"| {result['median']:.3f} s "
                f"| {result['mean']:.3f} s ± {result['stddev']:.3f} s "
                f"| {result['min']:.3f} s … {result['max']:.3f} s |"
            )
    lines += ["", "</details>", ""]

    args.out.write_text("\n".join(lines))
    print(f"wrote {args.out}", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
