import re
import urllib.request

URL = "https://raw.githubusercontent.com/jessedorrestijn2-bit/leashterm/main/cases/case2-network/hosted/status.txt"

def fetch(url):
    with urllib.request.urlopen(url, timeout=10) as response:
        return response.read().decode("utf-8")

text = fetch(URL)

statuses = {}
for name, status in re.findall(r"(?m)^Service ([A-D]):\s*(.+?)\s*$", text):
    statuses[name] = status

if len(statuses) < 4:
    match = re.search(r"https://gist\.githubusercontent\.com/\S+", text)
    if match:
        full_text = fetch(match.group(0).rstrip(")"))
        for name, status in re.findall(r"(?m)^Service ([A-D]):\s*(.+?)\s*$", full_text):
            statuses[name] = status

for name in "ABCD":
    print(f"Service {name}: {statuses.get(name, 'unknown')}")
