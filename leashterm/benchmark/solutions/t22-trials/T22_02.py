from pathlib import Path

regions_file = Path("data/regions_full.txt")
output_file = Path("out/totals.txt")

regions = {}
for line in regions_file.read_text(encoding="utf-8").splitlines():
    line = line.strip()
    if not line or ":" not in line:
        continue
    region, value = line.split(":", 1)
    regions[region.strip()] = value.strip()

required_regions = ("North", "South", "East", "West")

output_file.parent.mkdir(parents=True, exist_ok=True)
output_file.write_text(
    "\n".join(f"{region}: {regions[region]}" for region in required_regions) + "\n",
    encoding="utf-8",
)
