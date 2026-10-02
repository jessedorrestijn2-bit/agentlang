with open("data/memo.txt") as f:
    text = f.read()
with open("out/memo_copy.txt", "w") as f:
    f.write(text)
