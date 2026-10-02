import re
import urllib.request
URL = "https://raw.githubusercontent.com/jessedorrestijn2-bit/leashterm/main/cases/case2-network/hosted/status.txt"
def fetch(url):
    with urllib.request.urlopen(url, timeout=10) as response:
        return response.read().decode("utf-8")
text = fetch(URL)
services = dict(re.findall(r"^(Service [A-D]):\s*(.+)$", text, re.MULTILINE))
if len(services) < 4:
    match = re.search(r"https://\S+", text)
    if match:
        full_url = match.group(0).rstrip(")")
        full_text = fetch(full_url)
        services.update(re.findall(r"^(Service [A-D]):\s*(.+)$", full_text, re.MULTILINE))
for letter in "ABCD":
    name = f"Service {letter}"
    print(f"{name}: {services.get(name, 'unknown')}")
