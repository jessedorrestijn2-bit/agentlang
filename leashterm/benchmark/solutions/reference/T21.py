with open("data/report.txt") as src, open("out/report_final.txt", "w") as dst:
    dst.write(src.read())
