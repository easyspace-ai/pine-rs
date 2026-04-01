#!/usr/bin/env python3
"""
Compare strategy output against golden file.
Usage: python3 compare_strategy.py <golden_file> < <actual_output>
"""
import json
import sys

def main():
    if len(sys.argv) < 2:
        print("Usage: python3 compare_strategy.py <golden_file>", file=sys.stderr)
        sys.exit(1)

    golden_path = sys.argv[1]

    try:
        with open(golden_path, 'r') as f:
            golden = json.load(f)
    except FileNotFoundError:
        print(f"Golden file not found: {golden_path}", file=sys.stderr)
        sys.exit(1)
    except json.JSONDecodeError as e:
        print(f"Invalid golden file JSON: {e}", file=sys.stderr)
        sys.exit(1)

    try:
        actual = json.load(sys.stdin)
    except json.JSONDecodeError as e:
        print(f"Invalid actual output JSON: {e}", file=sys.stderr)
        sys.exit(1)

    # Compare strategy results
    golden_strategy = golden.get('strategy', {})
    actual_strategy = actual.get('strategy', {})

    errors = []

    # Check basic fields
    if golden_strategy.get('name') != actual_strategy.get('name'):
        errors.append(f"Strategy name mismatch: expected {golden_strategy.get('name')}, got {actual_strategy.get('name')}")

    # Check entries count (allow actual to have more entries than golden for demo)
    golden_entries = len(golden_strategy.get('entries', []))
    actual_entries = len(actual_strategy.get('entries', []))
    if actual_entries < golden_entries:
        errors.append(f"Too few entries: expected at least {golden_entries}, got {actual_entries}")

    # Check exits count
    golden_exits = len(golden_strategy.get('exits', []))
    actual_exits = len(actual_strategy.get('exits', []))
    if actual_exits < golden_exits:
        errors.append(f"Too few exits: expected at least {golden_exits}, got {actual_exits}")

    # Check position direction
    if golden_strategy.get('position_direction') != actual_strategy.get('position_direction'):
        errors.append(f"Position direction mismatch: expected {golden_strategy.get('position_direction')}, got {actual_strategy.get('position_direction')}")

    if errors:
        print("Strategy comparison failed:", file=sys.stderr)
        for err in errors:
            print(f"  - {err}", file=sys.stderr)
        sys.exit(1)

    print("Strategy output matches golden file")
    sys.exit(0)

if __name__ == "__main__":
    main()
