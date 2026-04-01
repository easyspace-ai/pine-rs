#!/usr/bin/env python3
"""比较实际输出与黄金 CSV，误差阈值 1e-8"""

import csv
import json
import math
import re
import sys

MARKET_COLUMNS = {"time", "open", "high", "low", "close", "volume", "bar_index"}
LEGACY_VALUE_COLUMNS = {"value"}


def normalize_name(name):
    return re.sub(r"[^a-z0-9]", "", name.lower())


def parse_actual_value(value):
    if value is None:
        return None
    if isinstance(value, (int, float)):
        if isinstance(value, float) and math.isnan(value):
            return None
        return float(value)
    return None


def load_expected(expected_csv):
    with open(expected_csv, newline="") as f:
        reader = csv.DictReader(f)
        fieldnames = reader.fieldnames or []

        if {"bar_index", "value"}.issubset(fieldnames):
            expected = []
            for row in reader:
                raw = row["value"].strip()
                expected.append(None if raw in ("", "na") else float(raw))
            return {"value": expected}

        value_columns = [name for name in fieldnames if name not in MARKET_COLUMNS]
        if not value_columns:
            raise ValueError(f"黄金 CSV 缺少可比较列: {expected_csv}")

        rows = list(reader)
        expected_by_column = {}
        for target_column in value_columns:
            expected = []
            for row in rows:
                raw = row.get(target_column, "").strip()
                expected.append(None if raw in ("", "na") else float(raw))
            expected_by_column[target_column] = expected

        return expected_by_column


def select_actual_series(actual_json, target_column):
    outputs = actual_json.get("outputs", {})
    if not outputs:
        raise ValueError("实际输出缺少 outputs 字段")

    normalized_target = normalize_name(target_column)
    normalized_outputs = {normalize_name(name): name for name in outputs}

    if normalized_target in normalized_outputs:
        return normalized_outputs[normalized_target], outputs[normalized_outputs[normalized_target]]

    if "output" in normalized_outputs:
        name = normalized_outputs["output"]
        return name, outputs[name]

    non_market_series = [
        name for name in outputs
        if normalize_name(name) not in {"close", "open", "high", "low", "volume", "time"}
    ]
    if len(non_market_series) == 1:
        name = non_market_series[0]
        return name, outputs[name]

    raise ValueError(f"无法从实际输出中匹配黄金列 {target_column}，可用系列: {', '.join(outputs.keys())}")


def main():
    if len(sys.argv) < 2:
        print("用法: compare_golden.py <expected.csv>")
        sys.exit(1)

    expected_csv = sys.argv[1]
    actual_json = json.load(sys.stdin)

    expected_map = load_expected(expected_csv)
    total_errors = 0

    for target_column, expected_values in expected_map.items():
        actual_series_name, actual_values = select_actual_series(actual_json, target_column)
        errors = 0
        total = min(len(expected_values), len(actual_values))

        if len(expected_values) != len(actual_values):
            print(
                f"{actual_series_name}: 长度不一致: expected={len(expected_values)}, actual={len(actual_values)}"
            )
            errors += abs(len(expected_values) - len(actual_values))

        for idx in range(total):
            exp_val = expected_values[idx]
            act_val = parse_actual_value(actual_values[idx])

            if exp_val is None and act_val is None:
                continue
            if exp_val is None or act_val is None:
                print(
                    f"{actual_series_name} bar[{idx}]: expected {'na' if exp_val is None else exp_val}, "
                    f"got {'na' if act_val is None else act_val}"
                )
                errors += 1
                continue

            diff = abs(act_val - exp_val)
            if diff > 1e-8:
                print(
                    f"{actual_series_name} bar[{idx}]: diff={diff:.2e} "
                    f"(expected={exp_val}, got={act_val})"
                )
                errors += 1

        total_errors += errors
        if errors == 0:
            print(f"✓ 系列 {actual_series_name} 全部 {total} 个数值通过 (误差 < 1e-8)")
        else:
            print(f"✗ 系列 {actual_series_name} 有 {errors} 个数值未通过")

    sys.exit(0 if total_errors == 0 else 1)


if __name__ == "__main__":
    main()
