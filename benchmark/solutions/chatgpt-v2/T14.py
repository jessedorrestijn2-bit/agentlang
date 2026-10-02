import pathlib
pathlib.Path("data/out_pointer.txt").read_text()
pathlib.Path("out/status.txt").write_text("done")
