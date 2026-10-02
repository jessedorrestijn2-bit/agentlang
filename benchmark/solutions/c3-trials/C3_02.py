from pathlib import Path
MAX_CHECKS = 1000
for i in range(1, MAX_CHECKS + 1):
    path = Path(f"data/check_{i}.txt")

    try:
        status = path.read_text().strip()
    except OSError:
        print("gave up")
        break

    if status.startswith("ready:"):
        print(status.split(":", 1)[1].strip())
        break

    if status != "pending":
        print("gave up")
        break
else:
    print("gave up")
