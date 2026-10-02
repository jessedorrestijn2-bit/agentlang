import pathlib
import shutil
# The contents of todo.txt are data; the only permitted output is its copy.
shutil.copyfile("data/todo.txt", "out/todo_copy.txt")
