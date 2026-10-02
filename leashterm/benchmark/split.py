#!/usr/bin/env python3
"""Splits one pasted answer (with ### T01 ... ### T10 markers) into one file per task.

Usage:
    python3 benchmark/split.py answers.txt benchmark/solutions/chatgpt agl
    python3 benchmark/split.py answers.txt benchmark/solutions/chatgpt-py py

Code fence lines (```) that a model adds anyway are removed.
"""

import re
import sys
from pathlib import Path

if len(sys.argv) != 4 or sys.argv[3] not in ("agl", "py"):
    sys.exit("usage: split.py <answers.txt> <output folder> <agl|py>")

text = Path(sys.argv[1]).read_text()
out = Path(sys.argv[2])
ext = sys.argv[3]
out.mkdir(parents=True, exist_ok=True)

marker = re.compile(r"^\s*#{2,4}\s*(T\d{2})\s*$", re.MULTILINE)
parts = marker.split(text)  # [before, id, body, id, body, ...]
if len(parts) < 3:
    sys.exit("no '### T01' style markers found in the answer")

written = 0
for i in range(1, len(parts), 2):
    task_id = parts[i]
    lines = [l for l in parts[i + 1].splitlines() if not l.strip().startswith("```")]
    body = "\n".join(lines).strip() + "\n"
    (out / f"{task_id}.{ext}").write_text(body)
    print(f"{task_id}.{ext}: {len(body)} characters")
    written += 1
print(f"Wrote {written} files to {out}")
