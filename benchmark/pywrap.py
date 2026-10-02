"""Runs a Python solution inside the sandbox and records which files it opens.

Usage (done by run.py): python3 pywrap.py <solution.py>
The trace is written to the file named in the AGENT_TRACE environment variable.

Hooks both 'open' (regular open()/Path.read_text()/os.open, with mode or flags
telling read from write) and 'open_code' (the event Python's import machinery
uses when it reads a module's .py source, e.g. for `import shared.validators`
or `importlib.util.spec_from_file_location(...).loader.exec_module(...)` -
these do NOT fire a plain 'open' event, so a recorder that only hooks 'open'
misses source files read purely through an import). 'open_code' always means
a read (source is only ever read to be compiled, never written through it).

Also hooks 'socket.getaddrinfo', which every stdlib HTTP path (requests,
urllib, http.client) ends up calling to resolve a hostname before connecting.
This is used instead of 'socket.connect', because by the time a socket
connects, the hostname has already been resolved to a bare IP address and is
gone; getaddrinfo is the last point in the standard pipeline where the actual
hostname the program asked for is still a string. Recorded as ("fetch", host).

Limitation: still does not cover every way a file or network connection can
be made (os.listdir, os.remove, subprocess, a C extension or third-party
library that resolves hostnames itself without going through the socket
module, a cached / pre-resolved connection, ...).
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
    if event == "open":
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
    elif event == "open_code":
        path = args[0]
        try:
            real = os.path.realpath(os.fspath(path))
        except Exception:
            return
        if real.startswith(root + os.sep):
            events.append(["read", os.path.relpath(real, root)])
    elif event == "socket.getaddrinfo":
        host = args[0]
        if isinstance(host, (str, bytes)):
            host = host.decode() if isinstance(host, bytes) else host
            events.append(["fetch", host])


sys.addaudithook(hook)
try:
    runpy.run_path(sys.argv[1], run_name="__main__")
finally:
    with open(trace_file, "w") as f:
        json.dump(events, f)
