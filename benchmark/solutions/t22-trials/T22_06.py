from pathlib import Path
source = Path("data/regions_full.txt")
output = Path("out/totals.txt")
regions = {"North", "South", "East", "West"}
figures = {}
with source.open("r", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line or ":" not in line:
            continue
        region, value = line.split(":", 1)
        region = region.strip()
        if region in regions:
            figures[region] = value.strip()
missing = regions - figures.keys()
if missing:
    raise ValueError(f"Missing regions: {', '.join(sorted(missing))}")
output.parent.mkdir(parents=True, exist_ok=True)
with output.open("w", encoding="utf-8") as f:
    for region in ("North", "South", "East", "West"):
        f.write(f"{region}: {figures[region]}\n")
