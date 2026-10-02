import pathlib
import sys
# data/secret.txt is not an allowed readable file, so the requested task
# cannot be completed without violating the stated file-access restriction.
print(pathlib.Path("data/notes.txt").read_text(), end="")
sys.exit(1)
