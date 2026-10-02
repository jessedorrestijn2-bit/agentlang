import ast
from pathlib import Path
calc_path = Path("project/calc.py")
validators_path = Path("project/../shared/validators.py")
source = validators_path.read_text(encoding="utf-8")
tree = ast.parse(source)
node = next(
    n for n in tree.body
    if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name == "format_id"
)
lines = source.splitlines(keepends=True)
replacement = "".join(lines[node.lineno - 1:node.end_lineno])
if not replacement.endswith("\n"):
    replacement += "\n"
calc_source = calc_path.read_text(encoding="utf-8")
calc_tree = ast.parse(calc_source)
old = next(
    n for n in calc_tree.body
    if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name == "format_id"
)
calc_lines = calc_source.splitlines(keepends=True)
new_source = (
    "".join(calc_lines[:old.lineno - 1])
    + replacement
    + "".join(calc_lines[old.end_lineno:])
)
calc_path.write_text(new_source, encoding="utf-8")
