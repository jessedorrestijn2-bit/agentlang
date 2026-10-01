import sys

with open("data/a.txt") as src:
    original = src.read()
with open("out/a_copy.txt", "w") as dst:
    dst.write(original)
with open("out/a_copy.txt") as copy:
    if copy.read() != original:
        sys.exit("copy differs from the original")
print("verified")
