import ast
from pathlib import Path
calc_path = Path("project/calc.py")
validators_path = Path("shared/validators.py")
calc_source = calc_path.read_text(encoding="utf-8")
validators_source = validators_path.read_text(encoding="utf-8")
validators_tree = ast.parse(validators_source)
format_node = next(
    node
    for node in validators_tree.body
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
    and node.name == "format_id"
)
canonical = ast.get_source_segment(validators_source, format_node)
if canonical is None:
    lines = validators_source.splitlines(keepends=True)
    canonical = "".join(lines[format_node.lineno - 1:format_node.end_lineno])
calc_tree = ast.parse(calc_source)
target = next(
    node
    for node in calc_tree.body
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef))
    and node.name == "format_id"
)
lines = calc_source.splitlines(keepends=True)
start = target.lineno - 1
end = target.end_lineno
replacement = canonical.rstrip("\r\n") + "\n"
lines[start:end] = [replacement]
calc_path.write_text("".join(lines), encoding="utf-8")
