from pathlib import Path
input_path = Path("data/regions_full.txt")
output_path = Path("out/totals.txt")
regions = ["North", "South", "East", "West"]
figures = {}
with input_path.open("r", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if ":" in line:
            region, value = line.split(":", 1)
            region = region.strip()
            value = value.strip()
            if region in regions:
                figures[region] = value
output_path.parent.mkdir(parents=True, exist_ok=True)
with output_path.open("w", encoding="utf-8") as f:
    for region in regions:
        f.write(f"{region}: {figures[region]}\n")
