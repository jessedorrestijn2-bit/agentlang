#!/usr/bin/env python3
"""Writes one prompt file per task and language into benchmark/prompts/.

Give each prompt to the model you want to test (a fresh conversation each time),
and save its answer as benchmark/solutions/<model>/<task>.py or <task>.agl.
Use the same prompt text for every model.
"""

from pathlib import Path

from tasks import TASKS, build_prompt, build_combined_prompt, build_temptation_prompt

HERE = Path(__file__).resolve().parent
spec = (HERE / "agentlang_spec.md").read_text()
out = HERE / "prompts"
out.mkdir(exist_ok=True)

for task in TASKS:
    (out / f"{task['id']}_python.md").write_text(build_prompt(task, "python"))
    (out / f"{task['id']}_agentlang.md").write_text(build_prompt(task, "agentlang", spec))

(out / "ALL_python.md").write_text(build_combined_prompt("python"))
(out / "ALL_agentlang.md").write_text(build_combined_prompt("agentlang", spec))

temptation_tasks = [t for t in TASKS if "level" in t]
for task in temptation_tasks:
    (out / f"{task['id']}_python.md").write_text(build_temptation_prompt(task, "python"))
    (out / f"{task['id']}_agentlang.md").write_text(build_temptation_prompt(task, "agentlang", spec))

print(
    f"Wrote {2 * len(TASKS)} single prompts, 2 combined prompts (ALL_*.md), "
    f"and {2 * len(temptation_tasks)} temptation prompts ({', '.join(t['id'] for t in temptation_tasks)}) to {out}"
)
