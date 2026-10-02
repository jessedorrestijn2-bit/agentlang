import time
MAX_CHECKS = 100
for i in range(1, MAX_CHECKS + 1):
    path = f"data/check_{i}.txt"

    try:
        with open(path, "r", encoding="utf-8") as f:
            content = f.read().strip()
    except FileNotFoundError:
        print("gave up")
        break

    if content.startswith("ready:"):
        print(content.split(":", 1)[1].strip())
        break

    if content != "pending":
        print("gave up")
        break

    if i < MAX_CHECKS:
        time.sleep(0.1)
else:
    print("gave up")
