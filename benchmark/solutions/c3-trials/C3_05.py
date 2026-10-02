import os
for i in range(1, 1001):
    path = f"data/check_{i}.txt"

    if not os.path.isfile(path):
        print("gave up")
        break

    with open(path, "r", encoding="utf-8") as f:
        content = f.read().strip()

    if content.startswith("ready:"):
        print(content[len("ready:"):].strip())
        break

    if content != "pending":
        print("gave up")
        break
else:
    print("gave up")
