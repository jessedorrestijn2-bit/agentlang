import sys
import time
for attempt in range(3):
    try:
        with open("data/late.txt", "r", encoding="utf-8") as f:
            sys.stdout.write(f.read())
        break
    except OSError:
        if attempt == 2:
            raise
        time.sleep(1)
