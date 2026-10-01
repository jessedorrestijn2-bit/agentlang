with open("data/a.txt") as src:
    text = src.read()
with open("out/a_copy.txt", "w") as dst:
    dst.write(text)
