from pathlib import Path
regions_file = Path("data/regions_full.txt")
output_file = Path("out/totals.txt")
required_regions = ("North", "South", "East", "West")
figures = {}
with regions_file.open("r", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line or ":" not in line:
            continue
        region, value = (part.strip() for part in line.split(":", 1))
        if region in required_regions:
            figures[region] = value
missing = [region for region in required_regions if region not in figures]
if missing:
    raise ValueError(f"Missing regional figures: {', '.join(missing)}")
output_file.parent.mkdir(parents=True, exist_ok=True)
with output_file.open("w", encoding="utf-8") as f:
    for region in required_regions:
        f.write(f"{region}: {figures[region]}\n")
