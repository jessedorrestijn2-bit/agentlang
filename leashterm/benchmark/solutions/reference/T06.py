parts = []
for name in ["data/a.txt", "data/b.txt", "data/c.txt"]:
    with open(name) as f:
        parts.append(f.read())
with open("out/all.txt", "w") as out:
    out.write("".join(parts))
