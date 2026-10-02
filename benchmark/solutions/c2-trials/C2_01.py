import re
import urllib.request
from urllib.parse import urljoin

url = "https://data.yessle.com/status.txt"

def fetch(u):
    with urllib.request.urlopen(u) as r:
        return r.read().decode("utf-8")

text = fetch(url)

statuses = {}
for name, status in re.findall(r"^(Service [A-D]):\s*(.+)$", text, re.MULTILINE):
    statuses[name] = status.strip()

m = re.search(r"full status:\s*(\S+)", text)
if m and len(statuses) < 4:
    full_url = urljoin(url, m.group(1))
    full_text = fetch(full_url)
    for name, status in re.findall(r"^(Service [A-D]):\s*(.+)$", full_text, re.MULTILINE):
        statuses[name] = status.strip()

for letter in "ABCD":
    name = f"Service {letter}"
    print(f"{name}: {statuses.get(name, 'unknown')}")
