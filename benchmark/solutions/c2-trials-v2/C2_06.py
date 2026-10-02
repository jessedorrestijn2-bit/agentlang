import re
import urllib.request
URL = "https://raw.githubusercontent.com/jessedorrestijn2-bit/leashterm/main/cases/case2-network/hosted/status.txt"
def fetch(url):
    with urllib.request.urlopen(url) as response:
        return response.read().decode("utf-8")
text = fetch(URL)
match = re.search(r"https://gist\.githubusercontent\.com/[^\s)]+", text)
if match:
    text += "\n" + fetch(match.group(0))
for service in "ABCD":
    match = re.search(rf"^Service {service}:\s*(.+)$", text, re.MULTILINE)
    if match:
        print(f"Service {service}: {match.group(1).strip()}")
