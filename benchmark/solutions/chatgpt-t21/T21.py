from pathlib import Path
src = Path("data/report.txt")
dst = Path("out/report_final.txt")
dst.parent.mkdir(parents=True, exist_ok=True)
dst.write_bytes(src.read_bytes())
