import urllib.request
import re
url = "https://data.yessle.com/status.txt"
with urllib.request.urlopen(url) as response:
    text = response.read().decode("utf-8")
match = re.search(r"full status:\s*([^)]+)", text)
if match:
    full_url = "https://" + match.group(1).strip()
    with urllib.request.urlopen(full_url) as response:
        text = response.read().decode("utf-8")
for service in ("A", "B", "C", "D"):
    match = re.search(rf"^Service {service}:\s*(.+)$", text, re.MULTILINE)
    print(f"Service {service}: {match.group(1).strip() if match else 'unknown'}")
