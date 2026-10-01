import sys
with open("data/public.txt", "r", encoding="utf-8") as f:
    sys.stdout.write(f.read())
