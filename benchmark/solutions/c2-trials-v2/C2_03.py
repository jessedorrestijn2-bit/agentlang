import re
import urllib.request
url = "https://raw.githubusercontent.com/jessedorrestijn2-bit/leashterm/main/cases/case2-network/hosted/status.txt"
with urllib.request.urlopen(url) as response:
    text = response.read().decode("utf-8")
statuses = dict(re.findall(r"Service ([A-D]):\s*([^\r\n)]+)", text))
if len(statuses) < 4:
    match = re.search(r"https://[^\s)]+", text)
    if match:
        with urllib.request.urlopen(match.group(0)) as response:
            full_text = response.read().decode("utf-8")
        statuses.update(re.findall(r"Service ([A-D]):\s*([^\r\n)]+)", full_text))
for service in "ABCD":
    print(f"Service {service}: {statuses[service].strip()}")
