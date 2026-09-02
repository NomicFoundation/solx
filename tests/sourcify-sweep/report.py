#!/usr/bin/env python3
"""Render the Sourcify sweep report from run.py's JSONL results.

Accepts one or more result files (shard outputs may simply be concatenated)
and writes markdown: the outcome table, candidate wall-time percentiles, the
outcome split per corpus trait, and the failure census — candidate-only
failures ranked by normalised signature with example contract ids, so the
ranked list doubles as the frontend's feature-gap backlog.
"""

import argparse
import collections
import json
import pathlib
import statistics
import sys

TOP_SIGNATURES = 40
EXAMPLES = 3
OUTCOME_ORDER = ["ok", "solx-fail", "both-fail", "timeout"]
OUTCOME_MEANING = {
    "ok": "compiles: no errors and bytecode for the target source",
    "solx-fail": "candidate fails, baseline compiles (or no baseline given)",
    "both-fail": "candidate and baseline both fail: corpus artifact or documented solx limitation",
    "timeout": "candidate exceeded the per-contract timeout",
}


def load(paths):
    meta, results = {}, []
    for path in paths:
        with path.open() as handle:
            for line in handle:
                if not line.strip():
                    continue
                record = json.loads(line)
                if "meta" in record:
                    meta.update(record["meta"])
                else:
                    results.append(record)
    return meta, results


def percentile(values, fraction):
    values = sorted(values)
    return values[min(len(values) - 1, int(len(values) * fraction))]


def census(results, title, lines):
    """Rank failures by (kind, signature); list counts and example ids."""
    buckets = collections.defaultdict(list)
    for result in results:
        leg = result["solx"]
        buckets[(leg["kind"], leg.get("signature", ""))].append(result["id"])
    if not buckets:
        return
    lines += [f"### {title} ({len(results)})", ""]
    by_kind = collections.Counter(kind for kind, _ in buckets for _ in buckets[(kind, _)])
    lines += ["kinds: " + ", ".join(f"`{kind}` {count}" for kind, count in by_kind.most_common()), ""]
    lines += ["| count | kind | signature | examples |", "|---|---|---|---|"]
    ranked = sorted(buckets.items(), key=lambda item: (-len(item[1]), item[0]))
    for (kind, sig), ids in ranked[:TOP_SIGNATURES]:
        examples = ", ".join(f"`{i}`" for i in sorted(ids)[:EXAMPLES])
        lines.append(f"| {len(ids)} | {kind} | {sig.replace('|', '\\|')} | {examples} |")
    if len(ranked) > TOP_SIGNATURES:
        rest = sum(len(ids) for _, ids in ranked[TOP_SIGNATURES:])
        lines.append(f"| {rest} | | *{len(ranked) - TOP_SIGNATURES} further signatures* | |")
    lines.append("")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--results", required=True, nargs="+", type=pathlib.Path, help="run.py --out file(s)")
    parser.add_argument("--out", type=pathlib.Path, help="markdown report path [default: stdout]")
    parser.add_argument("--title", default="Sourcify sweep")
    args = parser.parse_args()

    meta, results = load(args.results)
    if not results:
        sys.exit("no results found")
    total = len(results)
    outcomes = collections.Counter(result["outcome"] for result in results)

    lines = [f"## {args.title}", ""]
    for name in ("solx", "baseline"):
        if name in meta:
            lines.append(f"- {name}: `{meta[name]}`")
    lines += [f"- contracts: {total}", ""]

    lines += ["| outcome | count | share | meaning |", "|---|---|---|---|"]
    for outcome in OUTCOME_ORDER + sorted(set(outcomes) - set(OUTCOME_ORDER)):
        if outcomes[outcome]:
            lines.append(f"| {outcome} | {outcomes[outcome]} | {outcomes[outcome] / total:.2%} | {OUTCOME_MEANING.get(outcome, '')} |")
    lines.append("")

    ok_walls = [result["solx"]["wall"] for result in results if result["outcome"] == "ok"]
    if ok_walls:
        lines += [
            "candidate wall time over `ok` contracts: "
            f"median {statistics.median(ok_walls):.2f}s, p90 {percentile(ok_walls, 0.9):.2f}s, "
            f"p99 {percentile(ok_walls, 0.99):.2f}s, max {max(ok_walls):.2f}s, total {sum(ok_walls):.0f}s",
            "",
        ]

    lines += ["### Outcome by corpus trait", "", "| trait | contracts | ok | solx-fail | both-fail | timeout |", "|---|---|---|---|---|---|"]
    trait_rows = [
        ("single source", lambda t: t["n_sources"] == 1),
        ("multiple sources", lambda t: t["n_sources"] > 1),
        ("remappings", lambda t: t["remappings"]),
        ("viaIR", lambda t: t["via_ir"]),
        ("mentions `assembly`", lambda t: t["assembly"]),
        ("explicit evmVersion", lambda t: t["evm_version"] is not None),
    ]
    for label, predicate in trait_rows:
        subset = [result for result in results if predicate(result["traits"])]
        if not subset:
            continue
        counter = collections.Counter(result["outcome"] for result in subset)
        cells = " | ".join(f"{counter[o]} ({counter[o] / len(subset):.0%})" for o in OUTCOME_ORDER)
        lines.append(f"| {label} | {len(subset)} | {cells} |")
    lines.append("")

    census([r for r in results if r["outcome"] == "solx-fail"], "Candidate-only failures", lines)
    census([r for r in results if r["outcome"] == "both-fail"], "Failures shared with the baseline", lines)
    timeouts = [r for r in results if r["outcome"] == "timeout"]
    if timeouts:
        lines += [f"### Timeouts ({len(timeouts)})", "", ", ".join(f"`{r['id']}`" for r in sorted(timeouts, key=lambda r: r["id"])[:20]), ""]

    report = "\n".join(lines)
    if args.out:
        args.out.write_text(report)
    else:
        print(report)
    return 0


if __name__ == "__main__":
    sys.exit(main())
