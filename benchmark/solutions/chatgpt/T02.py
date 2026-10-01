with open("data/a.txt", "rb") as src, open("out/a_copy.txt", "wb") as dst:
    dst.write(src.read())
