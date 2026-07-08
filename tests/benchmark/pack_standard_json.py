#!/usr/bin/env python3
"""Pack a directory of Solidity sources into a solc standard-JSON input.

Seed-fixture generator for the compile-time benchmark: keys every *.sol file
by its path relative to --root (optionally prefixed), which resolves relative
imports as long as the tree is self-contained. Long-term, fixtures should come
from hardhat's `bench:dump-standard-json` (the exact input solx receives);
this packer only bootstraps the corpus until those dumps are published.
"""

import argparse
import json
import pathlib
import sys

OUTPUT_SELECTION = [
    "abi",
    "metadata",
    "evm.methodIdentifiers",
    "evm.bytecode.object",
    "evm.deployedBytecode.object",
]


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", required=True, type=pathlib.Path)
    parser.add_argument("--prefix", default="", help="key prefix, e.g. 'contracts/'")
    parser.add_argument("--out", required=True, type=pathlib.Path)
    parser.add_argument("--exclude", action="append", default=[],
                        help="path substring to skip (repeatable)")
    parser.add_argument("--remapping", action="append", default=[],
                        help="settings.remappings entry, e.g. 'solmate/=lib/solmate/' (repeatable)")
    args = parser.parse_args()

    sources = {}
    for path in sorted(args.root.rglob("*.sol")):
        rel = path.relative_to(args.root).as_posix()
        if any(pattern in rel for pattern in args.exclude):
            continue
        sources[args.prefix + rel] = {"content": path.read_text()}

    if not sources:
        print(f"no .sol files under {args.root}", file=sys.stderr)
        return 1

    standard_json = {
        "language": "Solidity",
        "sources": sources,
        "settings": {
            "outputSelection": {"*": {"*": OUTPUT_SELECTION}},
        },
    }
    if args.remapping:
        standard_json["settings"]["remappings"] = args.remapping
    args.out.write_text(json.dumps(standard_json, indent=1, sort_keys=True) + "\n")
    print(f"{args.out}: {len(sources)} sources, {args.out.stat().st_size // 1024} KiB")
    return 0


if __name__ == "__main__":
    sys.exit(main())
