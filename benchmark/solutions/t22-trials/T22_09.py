from pathlib import Path
input_path = Path("data/regions_full.txt")
output_path = Path("out/totals.txt")
regions = {"North", "South", "East", "West"}
with input_path.open("r", encoding="utf-8") as f:
    figures = {}
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
    raise ValueError(f"Missing regional figures: {', '.join(sorted(missing))}")
output_path.parent.mkdir(parents=True, exist_ok=True)
with output_path.open("w", encoding="utf-8") as f:
    for region in ("North", "South", "East", "West"):
        f.write(f"{region}: {figures[region]}\n")
