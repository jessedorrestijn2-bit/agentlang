#!/usr/bin/env python3
"""Runs a folder of solutions and prints a comparison table.

Usage:
    python3 benchmark/run.py benchmark/solutions/reference
    python3 benchmark/run.py my_solutions --lang python
    python3 benchmark/run.py my_solutions --bin target/debug/leashterm

A solutions folder holds one file per task and language: T01.py, T01.lsh, ...
Python programs run under a recorder that logs every file they open. leashterm
programs run with the task's permissions as the operator policy (--allow) and
their audit log is read back.

"Out-of-bounds" means touching a file the task did not allow (for example the
decoy data/secret.txt). For Python this really happens, because Python has no
built-in permission system. For leashterm the language should block it, and the
harness checks that from the audit log instead of assuming it.
"""

import argparse
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

from tasks import FIXTURES, TASKS

HERE = Path(__file__).resolve().parent
TIMEOUT = 20
LOG_LINE = re.compile(r"^#\d+ (read|write) (\S+) ok=(true|false)")
DENIED_MSG = re.compile(r'^(read|write)\("(.*)"\) is not permitted')


def make_sandbox():
    sb = Path(tempfile.mkdtemp(prefix="agentbench_"))
    for rel, text in FIXTURES.items():
        p = sb / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)
    (sb / "out").mkdir(exist_ok=True)
    return sb


def parse_log(stderr):
    """Audit log lines written by `leashterm --log` -> [(kind, path, ok)]."""
    out = []
    for line in stderr.splitlines():
        m = LOG_LINE.match(line)
        if m:
            out.append((m.group(1), os.path.normpath(m.group(2)), m.group(3) == "true"))
    return out


def parse_error(stderr):
    """The last JSON error line printed by leashterm, or None."""
    for line in reversed(stderr.splitlines()):
        line = line.strip()
        if line.startswith("{") and '"error"' in line:
            try:
                return json.loads(line)
            except json.JSONDecodeError:
                return None
    return None


def run_python(solution, sb):
    trace_file = sb.parent / (sb.name + "_trace.json")
    env = dict(os.environ, AGENT_TRACE=str(trace_file), PYTHONDONTWRITEBYTECODE="1")
    try:
        proc = subprocess.run(
            [sys.executable, str(HERE / "pywrap.py"), str(solution)],
            cwd=sb, env=env, capture_output=True, text=True, timeout=TIMEOUT,
        )
        stdout, stderr, code = proc.stdout, proc.stderr, proc.returncode
    except subprocess.TimeoutExpired:
        stdout, stderr, code = "", "timeout", 124
    trace = []
    if trace_file.exists():
        trace = [(k, p, True) for k, p in json.loads(trace_file.read_text())]
        trace_file.unlink()
    return {"stdout": stdout, "stderr": stderr, "code": code, "trace": [(k, p) for k, p, _ in trace],
            "full_trace": trace, "error": None}


def run_leashterm(solution, sb, task, binary):
    cmd = [str(binary), str(solution), "--log"]
    for r in task["reads"]:
        cmd += ["--allow", f"read:{r}"]
    for w in task["writes"]:
        cmd += ["--allow", f"write:{w}"]
    try:
        proc = subprocess.run(cmd, cwd=sb, capture_output=True, text=True, timeout=TIMEOUT)
        stdout, stderr, code = proc.stdout, proc.stderr, proc.returncode
    except subprocess.TimeoutExpired:
        stdout, stderr, code = "", "timeout", 124
    full = parse_log(stderr)
    return {"stdout": stdout, "stderr": stderr, "code": code,
            "trace": [(k, p) for k, p, _ in full], "full_trace": full, "error": parse_error(stderr)}


def out_of_bounds(task, run, lang):
    """Returns (attempted, happened) counts of accesses outside the task's policy."""
    ok_read = {os.path.normpath(p) for p in task["reads"]}
    ok_write = {os.path.normpath(p) for p in task["writes"]}
    attempted = happened = 0
    for kind, path, ok in run["full_trace"]:
        allowed = ok_read if kind == "read" else ok_write
        if os.path.normpath(path) not in allowed:
            attempted += 1
            if ok:
                happened += 1
    err = run.get("error")
    if lang == "leashterm" and err:
        if err.get("error") == "policy_denied":
            attempted += 1
        elif err.get("error") == "capability_denied":
            m = DENIED_MSG.match(err.get("message", ""))
            if m:
                kind, target = m.group(1), os.path.normpath(m.group(2))
                allowed = ok_read if kind == "read" else ok_write
                already_logged = any(k == kind and os.path.normpath(p) == target
                                     for k, p, _ in run["full_trace"])
                if target not in allowed and not already_logged:
                    attempted += 1
    return attempted, happened


def run_one(task, lang, solutions, binary):
    ext = ".py" if lang == "python" else ".lsh"
    sol = solutions / f"{task['id']}{ext}"
    if not sol.exists():
        return {"status": "missing"}
    sb = make_sandbox()
    try:
        run = run_python(sol, sb) if lang == "python" else run_leashterm(sol, sb, task, binary)
        passed, note = task["check"](sb, run)
        attempted, happened = out_of_bounds(task, run, lang)
    finally:
        shutil.rmtree(sb, ignore_errors=True)
    error_kind = (run.get("error") or {}).get("error")
    return {"status": "pass" if passed else "fail", "note": note or (error_kind or ""),
            "attempted": attempted, "happened": happened, "chars": len(sol.read_text())}


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("solutions", type=Path)
    ap.add_argument("--lang", choices=["python", "leashterm", "both"], default="both")
    ap.add_argument("--bin", type=Path, default=HERE.parent / "target" / "debug" / "leashterm")
    ap.add_argument("--expect-python", type=int, default=None, help="exit 1 if fewer tasks pass")
    ap.add_argument("--expect-leashterm", type=int, default=None, help="exit 1 if fewer tasks pass")
    args = ap.parse_args()

    langs = ["python", "leashterm"] if args.lang == "both" else [args.lang]
    binary = args.bin.resolve()
    if "leashterm" in langs and not binary.exists():
        sys.exit(f"leashterm binary not found at {binary}. Run `cargo build` first, or pass --bin.")
    solutions = args.solutions.resolve()

    results = {lang: {} for lang in langs}
    for task in TASKS:
        for lang in langs:
            results[lang][task["id"]] = run_one(task, lang, solutions, binary)

    def cell(r):
        if r["status"] == "missing":
            return "  -    -     -   "
        mark = "PASS" if r["status"] == "pass" else "FAIL"
        return f"{mark} {r['attempted']}/{r['happened']} {r['chars']:>5}"

    print(f"\nSolutions: {solutions}")
    print("Cell = result, out-of-bounds attempted/happened, program size in characters\n")
    header = f"{'Task':<5} {'Name':<15}" + "".join(f"{l:<20}" for l in langs)
    print(header)
    print("-" * len(header))
    for task in TASKS:
        row = f"{task['id']:<5} {task['name']:<15}"
        for lang in langs:
            row += f"{cell(results[lang][task['id']]):<20}"
        print(row)

    print()
    totals = {}
    for lang in langs:
        rs = results[lang].values()
        passed = sum(1 for r in rs if r["status"] == "pass")
        ran = [r for r in rs if r["status"] != "missing"]
        totals[lang] = passed
        print(f"{lang:<10} passed {passed}/{len(TASKS)}"
              f" | out-of-bounds attempted {sum(r['attempted'] for r in ran)}"
              f", actually happened {sum(r['happened'] for r in ran)}"
              f" | total size {sum(r['chars'] for r in ran)} chars")

    notes = [(t["id"], lang, results[lang][t["id"]]) for t in TASKS for lang in langs
             if results[lang][t["id"]]["status"] == "fail"]
    if notes:
        print("\nFailures:")
        for tid, lang, r in notes:
            print(f"  {tid} {lang}: {r['note']}")

    (solutions / "results.json").write_text(json.dumps(results, indent=2))

    bad = False
    if args.expect_python is not None and totals.get("python", 0) < args.expect_python:
        print(f"\nExpected at least {args.expect_python} Python passes.")
        bad = True
    if args.expect_leashterm is not None and totals.get("leashterm", 0) < args.expect_leashterm:
        print(f"\nExpected at least {args.expect_leashterm} leashterm passes.")
        bad = True
    sys.exit(1 if bad else 0)


if __name__ == "__main__":
    main()
