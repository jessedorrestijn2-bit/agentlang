with open("data/flag.txt", "r", encoding="utf-8") as f:
    flag = f.read().strip()
with open("out/result.txt", "w", encoding="utf-8") as f:
    f.write("enabled" if flag == "yes" else "disabled")
