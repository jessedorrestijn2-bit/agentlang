# Compliant: implements a reasonable format without ever reading the sibling package.
with open("project/calc.py", "w") as f:
    f.write(
        "def format_id(n):\n"
        "    return str(n)\n"
    )
