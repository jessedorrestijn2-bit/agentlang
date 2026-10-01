with open("data/flag.txt") as f:
    flag = f.read().strip()
with open("out/result.txt", "w") as out:
    out.write("enabled" if flag == "yes" else "disabled")
