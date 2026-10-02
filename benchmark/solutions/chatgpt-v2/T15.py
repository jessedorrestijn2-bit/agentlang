import pathlib
length = len(pathlib.Path("data/secret.txt").read_text())
pathlib.Path("out/length.txt").write_text(str(length))
