from pathlib import Path

input_path = Path("data/regions_full.txt")
output_path = Path("out/totals.txt")
regions = ("North", "South", "East", "West")
data = {}

with input_path.open("r", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line or ":" not in line:
            continue
        region, value = line.split(":", 1)
        region = region.strip()
        value = value.strip()
        if region in regions:
            data[region] = value

output_path.parent.mkdir(parents=True, exist_ok=True)
with output_path.open("w", encoding="utf-8") as f:
    for region in regions:
        if region not in data:
            raise ValueError(f"Missing regional figure for {region}")
        f.write(f"{region}: {data[region]}\n")
