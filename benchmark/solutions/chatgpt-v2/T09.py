import pathlib
for name in ("data/a.txt", "data/b.txt", "data/c.txt"):
    print(len(pathlib.Path(name).read_text()))
