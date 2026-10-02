"""Runs a Python solution inside the sandbox and records which files it opens.

Usage (done by run.py): python3 pywrap.py <solution.py>
The trace is written to the file named in the AGENT_TRACE environment variable.

Limitation: only file opens are recorded (open, io.open, os.open), not other
ways of touching files (os.listdir, os.remove, subprocess, ...).
"""

import json
import os
import runpy
import sys

trace_file = os.environ["AGENT_TRACE"]
root = os.path.realpath(os.getcwd())
events = []


def is_write(mode, flags):
    if isinstance(mode, str):
        return any(c in mode for c in "wax+")
    if isinstance(flags, int):
        bits = os.O_WRONLY | os.O_RDWR | os.O_CREAT | os.O_APPEND | os.O_TRUNC
        return bool(flags & bits)
    return False


def hook(event, args):
    if event != "open":
        return
    path = args[0]
    if isinstance(path, int):
        return
    try:
        real = os.path.realpath(os.fspath(path))
    except Exception:
        return
    if real.startswith(root + os.sep):
        rel = os.path.relpath(real, root)
        mode = args[1] if len(args) > 1 else None
        flags = args[2] if len(args) > 2 else None
        events.append(["write" if is_write(mode, flags) else "read", rel])


sys.addaudithook(hook)
try:
    runpy.run_path(sys.argv[1], run_name="__main__")
finally:
    with open(trace_file, "w") as f:
        json.dump(events, f)
