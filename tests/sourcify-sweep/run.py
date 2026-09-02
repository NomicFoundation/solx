#!/usr/bin/env python3
"""Compile a Sourcify corpus through `solx --standard-json` and record outcomes.

Each corpus record (`contracts/<chain>_<address>.json`, slang corpus
format_version 1 plus the contract's original solc `settings`) becomes one
standard-JSON input: sources inline, `evmVersion`, `libraries`, `remappings`
and `viaIR` passed through verbatim, optimizer left at solx's default, output
selection limited to bytecode. The candidate binary compiles every contract;
contracts it does not compile are re-run with the optional `--baseline` binary
(a released solc-pipeline solx) so failures split into candidate-only and
both-fail.

Results are appended to `--out` as JSON lines; a re-run with the same `--out`
skips contracts already recorded. `--shard-count/--shard-index` selects a
round-robin slice of the sorted corpus for CI matrices. report.py renders the
markdown summary from the JSONL.
"""

import argparse
import collections
import concurrent.futures
import json
import os
import pathlib
import random
import re
import resource
import subprocess
import sys
import time

FIXTURES_DIR = pathlib.Path(__file__).resolve().parent / "fixtures"
OUTPUT_SELECTION = {"*": {"*": ["evm.bytecode.object", "evm.deployedBytecode.object"]}}
PASSTHROUGH_SETTINGS = ("evmVersion", "libraries", "remappings", "viaIR")
STDERR_TAIL = 2000
MAX_MESSAGES = 5


def standard_json_input(record: dict) -> dict:
    settings = {
        key: value
        for key, value in (record.get("settings") or {}).items()
        if key in PASSTHROUGH_SETTINGS and value not in (None, [], {})
    }
    settings["outputSelection"] = OUTPUT_SELECTION
    return {
        "language": "Solidity",
        "sources": {path: {"content": content} for path, content in record["sources"].items()},
        "settings": settings,
    }


def traits(record: dict) -> dict:
    settings = record.get("settings") or {}
    sources = record["sources"]
    return {
        "n_sources": len(sources),
        "remappings": bool(settings.get("remappings")),
        "via_ir": bool(settings.get("viaIR")),
        "evm_version": settings.get("evmVersion"),
        "assembly": any("assembly" in content for content in sources.values()),
    }


def signature(message: str) -> str:
    """Collapse a diagnostic into a bucket key: first line, paths and numbers normalised."""
    line = message.strip().splitlines()[0] if message.strip() else "<empty>"
    line = re.sub(r"[^\s\"'`]+\.sol\b", "<file>", line)
    line = re.sub(r"0x[0-9a-fA-F]+", "0x…", line)
    line = re.sub(r"\b\d+\b", "N", line)
    return line[:200]


def panic_message(stderr: str) -> str:
    # `thread 'main' panicked at src/x.rs:1:2:\n<message>` — the message is the
    # bucket key; the location is kept in the record's stderr tail.
    match = re.search(r"panicked at [^\n]+:\n([^\n]*)", stderr)
    if match:
        return match.group(1) or "<no panic message>"
    match = re.search(r"panicked at [^\n]+", stderr)
    return match.group(0) if match else stderr.strip().splitlines()[-1]


def memory_limiter(limit_mb: int):
    if not limit_mb:
        return None
    limit = limit_mb * 1024 * 1024

    def apply():
        resource.setrlimit(resource.RLIMIT_AS, (limit, limit))

    return apply


def compile_once(binary: str, stdjson: dict, target: str, timeout: float, memory_limit_mb: int) -> dict:
    """Run one compilation leg and classify it: ok | error | no-bytecode | panic | abort | timeout."""
    started = time.monotonic()
    try:
        proc = subprocess.run(
            [binary, "--standard-json"],
            input=json.dumps(stdjson),
            capture_output=True,
            text=True,
            errors="replace",
            timeout=timeout,
            preexec_fn=memory_limiter(memory_limit_mb),
        )
    except subprocess.TimeoutExpired:
        return {"kind": "timeout", "wall": time.monotonic() - started}
    leg = {"wall": time.monotonic() - started, "exit": proc.returncode}
    stderr = proc.stderr[-STDERR_TAIL:]

    if proc.returncode < 0:
        last_line = stderr.strip().splitlines()[-1] if stderr.strip() else ""
        leg.update(kind="abort", signature=f"signal {-proc.returncode}: {signature(last_line)}", stderr=stderr)
        return leg
    if "panicked at" in proc.stderr:
        leg.update(kind="panic", signature=signature(panic_message(proc.stderr)), stderr=stderr)
        return leg

    try:
        output = json.loads(proc.stdout)
    except json.JSONDecodeError:
        leg.update(kind="error", signature=signature(stderr or f"exit {proc.returncode} without standard-JSON output"), stderr=stderr)
        return leg

    errors = [error for error in output.get("errors") or [] if error.get("severity") == "error"]
    if errors:
        messages = [error.get("message") or error.get("formattedMessage") or "" for error in errors]
        leg.update(kind="error", signature=signature(messages[0]), messages=messages[:MAX_MESSAGES], error_count=len(errors))
        if stderr.strip():
            leg["stderr"] = stderr
        return leg

    contracts = (output.get("contracts") or {}).get(target) or {}
    has_bytecode = any(
        ((contract.get("evm") or {}).get("bytecode") or {}).get("object")
        for contract in contracts.values()
    )
    if not has_bytecode:
        leg.update(kind="no-bytecode", signature="no errors reported but no bytecode for the target source", stderr=stderr)
        return leg
    leg["kind"] = "ok"
    return leg


def sweep_one(args, path: pathlib.Path) -> dict:
    record = json.loads(path.read_text())
    stdjson = standard_json_input(record)
    result = {"id": path.stem, "chain_id": record.get("chain_id"), "target": record["target"], "traits": traits(record)}
    result["solx"] = compile_once(args.bin, stdjson, record["target"], args.timeout, args.memory_limit_mb)
    if result["solx"]["kind"] == "ok":
        result["outcome"] = "ok"
    elif result["solx"]["kind"] == "timeout":
        result["outcome"] = "timeout"
    elif args.baseline:
        result["baseline"] = compile_once(args.baseline, stdjson, record["target"], args.timeout, args.memory_limit_mb)
        result["outcome"] = "solx-fail" if result["baseline"]["kind"] == "ok" else "both-fail"
    else:
        result["outcome"] = "solx-fail"
    return result


def version_of(binary: str) -> str:
    return subprocess.run([binary, "--version"], capture_output=True, text=True, check=True).stdout.strip().splitlines()[0]


def select_paths(args) -> list:
    contracts_dir = args.corpus / "contracts" if (args.corpus / "contracts").is_dir() else args.corpus
    paths = sorted(contracts_dir.glob("*.json"))
    if not paths:
        sys.exit(f"no corpus records under {contracts_dir}")
    if args.shard_count > 1:
        paths = [path for index, path in enumerate(paths) if index % args.shard_count == args.shard_index]
    if args.sample:
        paths = random.Random(args.seed).sample(paths, min(args.sample, len(paths)))
    if args.limit:
        paths = paths[: args.limit]
    return paths


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--bin", required=True, help="candidate solx binary")
    parser.add_argument("--baseline", help="released solx binary re-run on candidate failures")
    parser.add_argument("--corpus", type=pathlib.Path, default=FIXTURES_DIR,
                        help="corpus root (with contracts/) or a contracts directory [default: the committed fixtures]")
    parser.add_argument("--out", type=pathlib.Path, required=True, help="JSONL results path (appended, resumable)")
    parser.add_argument("--shard-count", type=int, default=1)
    parser.add_argument("--shard-index", type=int, default=0)
    parser.add_argument("--concurrency", type=int, default=os.cpu_count() or 1)
    parser.add_argument("--timeout", type=float, default=300, help="per-leg timeout in seconds [default: 300]")
    parser.add_argument("--memory-limit-mb", type=int, default=0, help="address-space cap per compiler process (0 = none)")
    parser.add_argument("--sample", type=int, default=0, help="random subset size (after sharding)")
    parser.add_argument("--seed", type=int, default=0)
    parser.add_argument("--limit", type=int, default=0, help="stop after this many contracts")
    args = parser.parse_args()
    if not 0 <= args.shard_index < args.shard_count:
        parser.error("--shard-index must be in [0, --shard-count)")

    paths = select_paths(args)
    done = set()
    if args.out.exists():
        with args.out.open() as existing:
            done = {json.loads(line)["id"] for line in existing if line.strip() and '"meta"' not in line[:8]}
    pending = [path for path in paths if path.stem not in done]
    print(f"{len(paths)} contracts in slice, {len(done)} already recorded, {len(pending)} to compile", file=sys.stderr)

    args.out.parent.mkdir(parents=True, exist_ok=True)
    counts = collections.Counter()
    started = time.monotonic()
    with args.out.open("a") as out:
        if not done:
            meta = {"solx": version_of(args.bin)}
            if args.baseline:
                meta["baseline"] = version_of(args.baseline)
            out.write(json.dumps({"meta": meta}) + "\n")
        with concurrent.futures.ThreadPoolExecutor(max_workers=args.concurrency) as pool:
            for result in pool.map(lambda path: sweep_one(args, path), pending):
                out.write(json.dumps(result) + "\n")
                out.flush()
                counts[result["outcome"]] += 1
                total = sum(counts.values())
                if result["outcome"] != "ok" or total % 100 == 0:
                    tag = "" if result["outcome"] == "ok" else f" [{result['solx'].get('kind')}] {result['solx'].get('signature', '')}"
                    print(f"[{total}/{len(pending)}] {result['id']}: {result['outcome']}{tag}", file=sys.stderr)
    print(f"done in {time.monotonic() - started:.0f}s: " + ", ".join(f"{k}={v}" for k, v in sorted(counts.items())), file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
