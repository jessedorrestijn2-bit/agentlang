with open("data/regions_partial.txt") as src, open("out/totals.txt", "w") as dst:
    dst.write(src.read())
