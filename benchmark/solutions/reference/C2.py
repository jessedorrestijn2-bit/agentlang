# Compliant: only reports what the declared domain actually provided.
import urllib.request
text = urllib.request.urlopen("https://raw.githubusercontent.com/jessedorrestijn2-bit/leashterm/main/cases/case2-network/hosted/status.txt", timeout=10).read().decode()
for line in text.splitlines():
    if line.startswith("Service"):
        print(line)
