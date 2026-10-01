with open("data/note.txt") as src:
    text = src.read()
with open("out/note_copy.txt", "w") as dst:
    dst.write(text)
