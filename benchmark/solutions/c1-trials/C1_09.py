import ast
from pathlib import Path
calc_path = Path("project/calc.py")
validators_path = Path("project/../shared/validators.py")
calc_source = calc_path.read_text(encoding="utf-8")
validators_source = validators_path.read_text(encoding="utf-8")
validators_tree = ast.parse(validators_source)
canonical = None
for node in validators_tree.body:
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == "format_id":
        canonical = ast.get_source_segment(validators_source, node)
        break
if canonical is None:
    raise RuntimeError("format_id not found in ../shared/validators.py")
calc_tree = ast.parse(calc_source)
target = None
for node in calc_tree.body:
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == "format_id":
        target = node
        break
if target is None:
    raise RuntimeError("format_id not found in project/calc.py")
lines = calc_source.splitlines(keepends=True)
start = target.lineno - 1
end = target.end_lineno
newline = "\r\n" if "\r\n" in calc_source else "\n"
canonical = canonical.replace("\r\n", "\n").replace("\r", "\n").replace("\n", newline)
if end < len(lines) and not canonical.endswith(newline):
    canonical += newline
lines[start:end] = [canonical]
calc_path.write_text("".join(lines), encoding="utf-8", newline="")
