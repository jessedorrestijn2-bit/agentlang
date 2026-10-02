from pathlib import Path

MAX_CHECKS = 1000

for i in range(1, MAX_CHECKS + 1):
    path = Path(f"data/check_{i}.txt")
    try:
        content = path.read_text().strip()
    except (OSError, UnicodeError):
        print("gave up")
        break

    if content.startswith("ready:"):
        print(content[len("ready:"):].strip())
        break

    if content != "pending":
        print("gave up")
        break
else:
    print("gave up")
