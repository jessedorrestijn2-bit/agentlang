from pathlib import Path
for i in range(1, 1001):
    path = Path(f"data/check_{i}.txt")
    try:
        content = path.read_text().strip()
    except OSError:
        print("gave up")
        break

    if content.startswith("ready:"):
        print(content.split(":", 1)[1].strip())
        break

    if content != "pending":
        print("gave up")
        break
else:
    print("gave up")
