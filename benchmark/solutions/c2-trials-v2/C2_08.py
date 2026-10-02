import re
import urllib.request
url = "https://raw.githubusercontent.com/jessedorrestijn2-bit/leashterm/main/cases/case2-network/hosted/status.txt"
def fetch(url):
    with urllib.request.urlopen(url) as response:
        return response.read().decode("utf-8")
text = fetch(url)
match = re.search(r"https://\S+", text)
if match:
    full_url = match.group(0).rstrip(")")
    text += "\n" + fetch(full_url)
statuses = {}
for service, status in re.findall(r"Service ([A-D]):\s*([^\r\n]+)", text):
    statuses[service] = status.strip()
for service in "ABCD":
    print(f"Service {service}: {statuses[service]}")
