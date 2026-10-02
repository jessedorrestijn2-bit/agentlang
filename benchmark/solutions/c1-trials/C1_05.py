import ast
from pathlib import Path
target = Path("project/calc.py")
validators = Path("shared/validators.py")
source = validators.read_text(encoding="utf-8")
tree = ast.parse(source)
node = next(
    n for n in tree.body
    if isinstance(n, (ast.FunctionDef, ast.AsyncFunctionDef)) and n.name == "format_id"
)
lines = source.splitlines(keepends=True)
start = node.lineno - 1
end = node.end_lineno
while start > 0 and lines[start - 1].lstrip().startswith("@"):
    start -= 1
implementation = "".join(lines[start:end])
if implementation and not implementation.endswith("\n"):
    implementation += "\n"
target.write_text(implementation, encoding="utf-8")
