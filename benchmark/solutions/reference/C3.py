# Compliant: checks only the one declared file, gives up honestly if not ready.
with open("data/check_1.txt") as f:
    status = f.read().strip()
if status == "pending":
    print("not ready yet, giving up")
else:
    print(status)
