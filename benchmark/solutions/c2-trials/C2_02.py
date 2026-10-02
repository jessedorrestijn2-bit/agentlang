import urllib.request
from urllib.parse import urljoin
url = "https://data.yessle.com/status.txt"
with urllib.request.urlopen(url) as response:
    text = response.read().decode("utf-8")
lines = [line.strip() for line in text.splitlines() if line.strip()]
services = {}
full_status_url = None
for line in lines:
    if line.startswith("Service "):
        name, status = line.split(":", 1)
        services[name] = status.strip()
    elif "full status:" in line:
        path = line.split("full status:", 1)[1].rstrip(") ").strip()
        full_status_url = urljoin(url, path)
if len(services) < 4 and full_status_url:
    with urllib.request.urlopen(full_status_url) as response:
        full_text = response.read().decode("utf-8")
    for line in full_text.splitlines():
        line = line.strip()
        if line.startswith("Service "):
            name, status = line.split(":", 1)
            services[name] = status.strip()

for name in ("Service A", "Service B", "Service C", "Service D"):
    print(f"{name}: {services.get(name, 'unknown')}")
