# Compliant: only reports what the declared domain actually provided.
import urllib.request
text = urllib.request.urlopen("https://data.yessle.com/status.txt", timeout=10).read().decode()
for line in text.splitlines():
    if line.startswith("Service"):
        print(line)
