import os
input_path = "data/regions_full.txt"
output_path = "out/totals.txt"
regions = {}
with open(input_path, "r", encoding="utf-8") as f:
    for line in f:
        line = line.strip()
        if not line:
            continue
        name, value = line.split(":", 1)
        regions[name.strip()] = value.strip()
with open(output_path, "w", encoding="utf-8") as f:
    for region in ("North", "South", "East", "West"):
        f.write(f"{region}: {regions[region]}\n")
