import ast
from pathlib import Path

target = Path("project/calc.py")
canonical = Path("shared/validators.py")

source = canonical.read_text()
tree = ast.parse(source)

node = next(
    n for n in tree.body
    if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name == "format_id"
)

lines = source.splitlines(keepends=True)
replacement = "".join(lines[node.lineno - 1:node.end_lineno])
target.write_text(replacement)
