#!/usr/bin/env python3
"""
Diff PineTS `builtin.json` keys against names inferred from `pine-stdlib` sources.
Writes `docs/BUILTIN_GAP_REPORT.md`. Optional; does not define Phase completion.
"""
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
PINETS_JSON = ROOT / "three/PineTS/docs/api-coverage/pinescript-v6/builtin.json"
OUT = Path(__file__).resolve().parents[1] / "docs/BUILTIN_GAP_REPORT.md"
STDLIB_SRC = ROOT / "crates/pine-stdlib/src"


def load_pinets_keys(path: Path) -> set[str]:
    data = json.loads(path.read_text(encoding="utf-8"))
    keys: set[str] = set()

    def walk(obj: object) -> None:
        if isinstance(obj, dict):
            for k, v in obj.items():
                keys.add(k)
                walk(v)

    walk(data)
    return keys


def load_rs_api_names() -> set[str]:
    names: set[str] = set()
    if not STDLIB_SRC.is_dir():
        return names
    for path in STDLIB_SRC.rglob("*.rs"):
        text = path.read_text(encoding="utf-8", errors="replace")
        for m in re.finditer(
            r'#\s*\[\s*pine_builtin\s*\(\s*([^]]+)\)\s*\]',
            text,
            re.DOTALL,
        ):
            inner = m.group(1)
            bn = re.search(r'name\s*=\s*"([^"]+)"', inner)
            ns = re.search(r'namespace\s*=\s*"([^"]+)"', inner)
            if bn and ns:
                names.add(f"{ns.group(1)}.{bn.group(1)}")
            elif bn:
                names.add(bn.group(1))
        collapsed = re.sub(r"\s+", " ", text)
        for m in re.finditer(
            r'FunctionMeta::new\(\s*"([^"]+)"\s*\)\s*\.\s*with_namespace\(\s*"([^"]+)"\s*\)',
            collapsed,
        ):
            names.add(f"{m.group(2)}.{m.group(1)}")
        for m in re.finditer(r'FunctionMeta::new\(\s*"([^"]+)"\s*\)(?!\s*\.\s*with_namespace)', collapsed):
            names.add(m.group(1))
    return names


def normalize_pinets_key(k: str) -> str:
    return k.replace("()", "")


def main() -> int:
    if not PINETS_JSON.exists():
        OUT.parent.mkdir(parents=True, exist_ok=True)
        OUT.write_text(
            f"# Builtin gap report\n\nMissing `{PINETS_JSON.relative_to(ROOT)}`.\n",
            encoding="utf-8",
        )
        print(f"Wrote {OUT.relative_to(ROOT)} (stub)")
        return 0

    pinets = {normalize_pinets_key(k) for k in load_pinets_keys(PINETS_JSON)}
    rs_names = load_rs_api_names()

    only_pinets = sorted(pinets - rs_names, key=str.lower)
    only_rs = sorted(rs_names - pinets, key=str.lower)

    lines = [
        "# Builtin gap report (PineTS manifest vs pine-stdlib heuristic)",
        "",
        "- Manifest: `three/PineTS/docs/api-coverage/pinescript-v6/builtin.json` (keys; `()` stripped)",
        "- pine-rs: `FunctionMeta::new` + `#[pine_builtin]` in `crates/pine-stdlib/src`",
        "",
        f"- PineTS keys: {len(pinets)}",
        f"- pine-rs names (approx): {len(rs_names)}",
        "",
        "## In PineTS, not matched in pine-rs heuristic (first 250)",
        "",
    ]
    for n in only_pinets[:250]:
        lines.append(f"- `{n}`")
    if len(only_pinets) > 250:
        lines.append(f"- … {len(only_pinets) - 250} more")
    lines += ["", "## In pine-rs heuristic, not in PineTS keys (first 120)", ""]
    for n in only_rs[:120]:
        lines.append(f"- `{n}`")
    if len(only_rs) > 120:
        lines.append(f"- … {len(only_rs) - 120} more")

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(f"Wrote {OUT.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
