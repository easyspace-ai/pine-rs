#!/usr/bin/env python3
"""
将 pine-cli 的 JSON 输出与 PineTS 风格兼容性文件对比。

PineTS expect：`{ "results": { "series_name": [ float | "na" ] } }`
pine-cli：`{ "outputs": { "plot_title": [ float, ... ] } }`（na 常为 nan）

用法示例：
  python3 scripts/compare_compat_expect.py \\
    --script tests/compatibility/cases/plot_close.pine \\
    --data tests/data/BTCUSDT_1h.csv \\
    --expect path/to.abs.expect.json \\
    --expect-key abs_native \\
    --actual-key plot
"""
from __future__ import annotations

import argparse
import json
import math
import subprocess
import sys
from pathlib import Path


def parse_cli_json(stdout: str) -> dict:
    start = stdout.find("{")
    if start == -1:
        raise ValueError("pine-cli 输出中未找到 JSON")
    return json.loads(stdout[start:])


def to_floats(seq: list) -> list[float | None]:
    out: list[float | None] = []
    for x in seq:
        if x is None or (isinstance(x, str) and x.lower() in ("", "na", "nan")):
            out.append(None)
        elif isinstance(x, (int, float)):
            xf = float(x)
            if math.isnan(xf):
                out.append(None)
            else:
                out.append(xf)
        else:
            raise TypeError(f"无法解析 expect 元素: {x!r}")
    return out


def close_enough(a: float | None, b: float | None, tol: float) -> bool:
    if a is None and b is None:
        return True
    if a is None or b is None:
        return False
    if math.isnan(a) and math.isnan(b):
        return True
    return abs(a - b) <= tol


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--script", required=True, type=Path)
    ap.add_argument("--data", required=True, type=Path)
    ap.add_argument("--expect", required=True, type=Path)
    ap.add_argument("--expect-key", required=True, help="expect['results'] 下的键")
    ap.add_argument("--actual-key", required=True, help="CLI JSON outputs 下的键（如 plot 名）")
    ap.add_argument("--tol", type=float, default=1e-8)
    ap.add_argument(
        "--root",
        type=Path,
        default=Path(__file__).resolve().parents[1],
        help="仓库根（用于 cargo run）",
    )
    args = ap.parse_args()
    root: Path = args.root

    exp_raw = json.loads(args.expect.read_text(encoding="utf-8"))
    results = exp_raw.get("results") or {}
    if args.expect_key not in results:
        print(
            f"expect 中无 results[{args.expect_key!r}]，键有: {list(results)[:20]}...",
            file=sys.stderr,
        )
        return 2
    expected = to_floats(results[args.expect_key])

    cmd = [
        "cargo",
        "run",
        "-p",
        "pine-cli",
        "--quiet",
        "--",
        "run",
        str(args.script),
        "--data",
        str(args.data),
        "--format",
        "json",
    ]
    r = subprocess.run(cmd, cwd=root, capture_output=True, text=True)
    if r.returncode != 0:
        print(r.stderr or r.stdout, file=sys.stderr)
        return r.returncode or 1

    data = parse_cli_json(r.stdout)
    if not data.get("success"):
        print(data.get("error", "run failed"), file=sys.stderr)
        return 1
    outputs = data.get("outputs") or {}
    if args.actual_key not in outputs:
        print(f"outputs 中无 {args.actual_key!r}，键有: {list(outputs)}", file=sys.stderr)
        return 2
    actual_raw = outputs[args.actual_key]
    actual: list[float | None] = []
    for x in actual_raw:
        if x is None or (isinstance(x, float) and math.isnan(x)):
            actual.append(None)
        else:
            actual.append(float(x))

    n = min(len(expected), len(actual))
    if len(expected) != len(actual):
        print(
            f"警告: 长度不一致 expect={len(expected)} actual={len(actual)}，只比较前 {n} 根 bar",
            file=sys.stderr,
        )

    bad = 0
    for i in range(n):
        if not close_enough(expected[i], actual[i], args.tol):
            bad += 1
            if bad <= 5:
                print(f"  bar {i}: expect={expected[i]!r} actual={actual[i]!r}", file=sys.stderr)

    if bad:
        print(f"失败: {bad} / {n} 根 bar 超出容差 {args.tol}", file=sys.stderr)
        return 1
    print(f"OK: {n} 根 bar 在容差 {args.tol} 内一致")
    return 0


if __name__ == "__main__":
    sys.exit(main())
