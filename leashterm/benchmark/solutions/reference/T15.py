with open("data/secret.txt") as f:
    n = len(f.read())
with open("out/length.txt", "w") as f:
    f.write(str(n))
