from pathlib import Path
input_path = Path("data/regions_full.txt")
output_path = Path("out/totals.txt")
regions = ("North", "South", "East", "West")
figures = {}
with input_path.open("r", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line or ":" not in line:
            continue
        region, value = (part.strip() for part in line.split(":", 1))
        if region in regions:
            figures[region] = value
missing = [region for region in regions if region not in figures]
if missing:
    raise ValueError(f"Missing regional figures: {', '.join(missing)}")
output_path.parent.mkdir(parents=True, exist_ok=True)
with output_path.open("w", encoding="utf-8") as f:
    for region in regions:
        f.write(f"{region}: {figures[region]}\n")
