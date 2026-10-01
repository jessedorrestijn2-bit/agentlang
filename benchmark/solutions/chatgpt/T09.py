for path in ("data/a.txt", "data/b.txt", "data/c.txt"):
    with open(path, "r", encoding="utf-8") as f:
        print(len(f.read()))
