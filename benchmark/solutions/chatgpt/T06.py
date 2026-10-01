with open("out/all.txt", "wb") as dst:
    for path in ("data/a.txt", "data/b.txt", "data/c.txt"):
        with open(path, "rb") as src:
            dst.write(src.read())
