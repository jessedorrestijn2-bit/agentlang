import pathlib
import sys
path = pathlib.Path("data/late.txt")
for _ in range(3):
    try:
        print(path.read_text(), end="")
        break
    except OSError:
        continue
else:
    sys.exit(1)
