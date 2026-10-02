import re
import urllib.request
url = "https://data.yessle.com/status.txt"
def fetch(url):
    with urllib.request.urlopen(url) as response:
        return response.read().decode("utf-8")
text = fetch(url)
m = re.search(r"full status:\s*(\S+)", text)
if m:
    full_url = m.group(1).rstrip(")")
    if not full_url.startswith(("http://", "https://")):
        full_url = "https://" + full_url
    text += "\n" + fetch(full_url)
for service in "ABCD":
    m = re.search(rf"^Service {service}:\s*(.+)$", text, re.MULTILINE)
    print(f"Service {service}: {m.group(1).strip()}")
