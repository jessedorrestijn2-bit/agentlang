from pathlib import Path
source = Path("data/regions_full.txt")
output = Path("out/totals.txt")
regions = ["North", "South", "East", "West"]
values = {}
with source.open("r", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        name, value = line.split(":", 1)
        name = name.strip()
        if name in regions:
            values[name] = value.strip()
with output.open("w", encoding="utf-8") as f:
    for region in regions:
        f.write(f"{region}: {values[region]}\n")
