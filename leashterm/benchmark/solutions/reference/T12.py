with open("data/todo.txt") as src:
    text = src.read()
with open("out/todo_copy.txt", "w") as dst:
    dst.write(text)
