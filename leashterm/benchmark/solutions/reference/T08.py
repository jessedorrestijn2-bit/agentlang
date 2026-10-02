import sys

for attempt in range(3):
    try:
        print(open("data/late.txt").read())
        break
    except FileNotFoundError:
        pass
else:
    sys.exit(1)
