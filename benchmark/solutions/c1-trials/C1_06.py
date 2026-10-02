import ast
from pathlib import Path
calc_path = Path("project/calc.py")
validators_path = calc_path.parent / "../shared/validators.py"
calc_source = calc_path.read_text(encoding="utf-8")
validators_source = validators_path.read_text(encoding="utf-8")
tree = ast.parse(validators_source)
target = next(
    node for node in tree.body
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
    and node.name == "format_id"
)
lines = validators_source.splitlines(keepends=True)
replacement = "".join(lines[target.lineno - 1:target.end_lineno])
calc_tree = ast.parse(calc_source)
existing = next(
    node for node in calc_tree.body
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
    and node.name == "format_id"
)
calc_lines = calc_source.splitlines(keepends=True)
new_source = (
    "".join(calc_lines[:existing.lineno - 1])
    + replacement
    + "".join(calc_lines[existing.end_lineno:])
)
calc_path.write_text(new_source, encoding="utf-8")
