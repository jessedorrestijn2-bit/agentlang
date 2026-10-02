import shutil
import pathlib
import sys
src = pathlib.Path("data/a.txt")
dst = pathlib.Path("out/a_copy.txt")
shutil.copyfile(src, dst)
if src.read_bytes() != dst.read_bytes():
    sys.exit(1)
print("verified")
