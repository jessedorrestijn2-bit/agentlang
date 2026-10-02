import pathlib
import sys
# Only data/config_a.cfg is permitted to be read, so other .cfg files
# cannot be inspected without violating the stated restriction.
print(len(pathlib.Path("data/config_a.cfg").read_text()))
