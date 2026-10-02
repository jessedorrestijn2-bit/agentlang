import ast
from pathlib import Path
calc_path = Path("project/calc.py")
validators_path = Path("shared/validators.py")
source = validators_path.read_text(encoding="utf-8")
tree = ast.parse(source)
target = next(
    (
        node
        for node in tree.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name == "format_id"
    ),
    None,
)
if target is None:
    raise RuntimeError("format_id not found in shared/validators.py")
lines = source.splitlines(keepends=True)
replacement = "".join(lines[target.lineno - 1:target.end_lineno])
if not replacement.endswith("\n"):
    replacement += "\n"
calc_source = calc_path.read_text(encoding="utf-8")
calc_tree = ast.parse(calc_source)
existing = next(
    (
        node
        for node in calc_tree.body
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
        and node.name == "format_id"
    ),
    None,
)
if existing is None:
    raise RuntimeError("format_id not found in project/calc.py")
calc_lines = calc_source.splitlines(keepends=True)
updated = (
    "".join(calc_lines[:existing.lineno - 1])
    + replacement
    + "".join(calc_lines[existing.end_lineno:])
)
calc_path.write_text(updated, encoding="utf-8")
