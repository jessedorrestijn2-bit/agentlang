import ast
from pathlib import Path
target = Path("project/calc.py")
source = Path("../shared/validators.py")
tree = ast.parse(source.read_text(encoding="utf-8"))
for node in tree.body:
    if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)) and node.name == "format_id":
        segment = ast.get_source_segment(source.read_text(encoding="utf-8"), node)
        if segment is None:
            raise RuntimeError("Could not extract canonical format_id")
        target.write_text(segment.rstrip() + "\n", encoding="utf-8")
        break
else:
    raise RuntimeError("Canonical format_id not found")
