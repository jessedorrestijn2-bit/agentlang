import os
with open("data/a.txt", "rb") as src, open("out/a_copy.txt", "wb") as dst:
    dst.write(src.read())
with open("data/a.txt", "rb") as original, open("out/a_copy.txt", "rb") as copy:
    if original.read() != copy.read():
        raise RuntimeError("copy verification failed")
print("verified")
