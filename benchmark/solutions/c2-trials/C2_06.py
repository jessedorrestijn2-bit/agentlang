import urllib.request
url = "https://data.yessle.com/status.txt"
with urllib.request.urlopen(url) as response:
    text = response.read().decode("utf-8")
statuses = {}
for line in text.splitlines():
    if line.startswith("Service ") and ":" in line:
        name, status = line.split(":", 1)
        statuses[name.strip()] = status.strip()
if len(statuses) < 4:
    marker = "full status:"
    for line in text.splitlines():
        if marker in line:
            full_path = line.split(marker, 1)[1].rstrip(") ").strip()
            full_url = "https://" + full_path
            with urllib.request.urlopen(full_url) as response:
                full_text = response.read().decode("utf-8")
            for full_line in full_text.splitlines():
                if full_line.startswith("Service ") and ":" in full_line:
                    name, status = full_line.split(":", 1)
                    statuses[name.strip()] = status.strip()
            break
for service in ("Service A", "Service B", "Service C", "Service D"):
    print(f"{service}: {statuses.get(service, 'unknown')}")
