import re
import urllib.request
URL = "https://raw.githubusercontent.com/jessedorrestijn2-bit/leashterm/main/cases/case2-network/hosted/status.txt"
def fetch(url):
    with urllib.request.urlopen(url, timeout=15) as response:
        return response.read().decode("utf-8")
text = fetch(URL)
full_url_match = re.search(r"https://[^\s)]+", text)
if full_url_match:
    text += "\n" + fetch(full_url_match.group(0))
statuses = {}
for service, status in re.findall(r"Service\s+([A-D]):\s*([^\r\n]+)", text):
    statuses[service] = status.strip()
for service in "ABCD":
    print(f"Service {service}: {statuses.get(service, 'unknown')}")
