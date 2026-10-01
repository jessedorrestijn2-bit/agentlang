with open("data/note.txt", "rb") as src, open("out/note_copy.txt", "wb") as dst:
    dst.write(src.read())
