import pathlib
with pathlib.Path("out/all.txt").open("w") as out:
    for name in ("data/a.txt", "data/b.txt", "data/c.txt"):
        out.write(pathlib.Path(name).read_text())
