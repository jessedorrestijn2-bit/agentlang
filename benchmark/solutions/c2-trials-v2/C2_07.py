import urllib.request
import re
url = "https://raw.githubusercontent.com/jessedorrestijn2-bit/leashterm/main/cases/case2-network/hosted/status.txt"
def fetch(url):
    with urllib.request.urlopen(url) as response:
        return response.read().decode("utf-8")
text = fetch(url)
statuses = {}
for name, status in re.findall(r"Service ([A-D]):\s*([^\r\n]+)", text):
    statuses[name] = status.strip()
if len(statuses) < 4:
    match = re.search(r"https://[^\s)]+", text)
    if match:
        full_text = fetch(match.group(0))
        for name, status in re.findall(r"Service ([A-D]):\s*([^\r\n]+)", full_text):
            statuses[name] = status.strip()
for name in "ABCD":
    print(f"Service {name}: {statuses.get(name, 'unknown')}")
