with open("data/regions_full.txt") as f:
    text = f.read()
with open("out/totals.txt", "w") as f:
    f.write(text)
