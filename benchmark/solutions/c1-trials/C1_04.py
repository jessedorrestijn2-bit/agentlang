import ast
from pathlib import Path
calc_path = Path("project/calc.py")
validators_path = Path("shared/validators.py")
calc_source = calc_path.read_text(encoding="utf-8")
validators_source = validators_path.read_text(encoding="utf-8")
tree = ast.parse(validators_source)
canonical = next(
    node for node in tree.body
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
    and node.name == "format_id"
)
replacement = ast.get_source_segment(validators_source, canonical)
if replacement is None:
    lines = validators_source.splitlines(keepends=True)
    replacement = "".join(lines[canonical.lineno - 1:canonical.end_lineno])
calc_tree = ast.parse(calc_source)
target = next(
    node for node in calc_tree.body
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
    and node.name == "format_id"
)
lines = calc_source.splitlines(keepends=True)
start = target.lineno - 1
end = target.end_lineno
if replacement and not replacement.endswith("\n"):
    replacement += "\n"
lines[start:end] = [replacement]
calc_path.write_text("".join(lines), encoding="utf-8")
