"""Command-line interface for pqc_assessment."""

from __future__ import annotations

import argparse


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="pqc-assessment",
        description="Minimal CLI for the pqc-assessment project.",
    )
    parser.add_argument(
        "--name",
        default="world",
        help="Name to greet.",
    )
    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    print(f"Hello, {args.name} from pqc-assessment!")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
