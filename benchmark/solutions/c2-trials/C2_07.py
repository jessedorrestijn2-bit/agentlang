import urllib.request
import re
url = "https://data.yessle.com/status.txt"
def fetch(url):
    with urllib.request.urlopen(url) as response:
        return response.read().decode("utf-8")
text = fetch(url)
statuses = dict(re.findall(r"^(Service [A-D]):\s*(.+)$", text, re.MULTILINE))
if len(statuses) < 4:
    match = re.search(r"full status:\s*([^) \r\n]+)", text)
    if match:
        full_url = "https://" + match.group(1)
        full_text = fetch(full_url)
        statuses.update(re.findall(r"^(Service [A-D]):\s*(.+)$", full_text, re.MULTILINE))
for name in ("Service A", "Service B", "Service C", "Service D"):
    print(f"{name}: {statuses[name]}")
