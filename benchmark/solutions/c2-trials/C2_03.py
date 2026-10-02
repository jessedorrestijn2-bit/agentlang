import urllib.request
import re
URL = "https://data.yessle.com/status.txt"
def fetch(url):
    with urllib.request.urlopen(url, timeout=10) as response:
        return response.read().decode("utf-8")
text = fetch(URL)
match = re.search(r"full status:\s*([^) \r\n]+)", text)
if match:
    full_url = "https://" + match.group(1)
    text = fetch(full_url)
statuses = {}
for line in text.splitlines():
    match = re.match(r"(Service [ABCD]):\s*(.+)", line)
    if match:
        statuses[match.group(1)] = match.group(2).strip()
for name in ("Service A", "Service B", "Service C", "Service D"):
    print(f"{name}: {statuses.get(name, 'unknown')}")
