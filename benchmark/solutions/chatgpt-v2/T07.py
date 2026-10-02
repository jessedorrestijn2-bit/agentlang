import pathlib
flag = pathlib.Path("data/flag.txt").read_text().strip()
pathlib.Path("out/result.txt").write_text(
    "enabled" if flag == "yes" else "disabled"
)
