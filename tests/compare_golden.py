#!/usr/bin/env python3
"""比较实际输出与黄金 CSV，误差阈值 1e-8"""
import sys, json, csv, math

def main():
    if len(sys.argv) < 2:
        print("用法: compare_golden.py <expected.csv>")
        sys.exit(1)
    
    expected_csv = sys.argv[1]
    actual_json = json.load(sys.stdin)
    
    expected = {}
    with open(expected_csv) as f:
        for row in csv.DictReader(f):
            expected[int(row['bar_index'])] = float(row['value']) if row['value'] != 'na' else None
    
    errors = 0
    for bar_idx, exp_val in expected.items():
        act_val = actual_json.get('plots', {}).get(str(bar_idx))
        if exp_val is None and act_val is None:
            continue
        if exp_val is None or act_val is None:
            print(f"bar[{bar_idx}]: expected {'na' if exp_val is None else exp_val}, got {'na' if act_val is None else act_val}")
            errors += 1
            continue
        diff = abs(float(act_val) - exp_val)
        if diff > 1e-8:
            print(f"bar[{bar_idx}]: diff={diff:.2e} (expected={exp_val}, got={act_val})")
            errors += 1
    
    if errors == 0:
        print(f"✓ 全部 {len(expected)} 个数值通过 (误差 < 1e-8)")
        sys.exit(0)
    else:
        print(f"✗ {errors}/{len(expected)} 个数值超出误差阈值")
        sys.exit(1)

if __name__ == '__main__':
    main()
