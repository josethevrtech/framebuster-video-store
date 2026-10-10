import csv
import io
import math
import statistics
import subprocess
from pathlib import Path


def collect(root, name, command, env):
    with (
        (root / f"{name}.stdout").open("w") as stdout,
        (root / f"{name}.log").open("w") as stderr,
    ):
        result = subprocess.run(
            ["timeout", "900", *command],
            env=env,
            stdout=stdout,
            stderr=stderr,
            check=False,
        )
    lines = (root / f"{name}.stdout").read_text().splitlines()
    header = next((line for line in lines if line.startswith("run,request,")), None)
    rows = []
    if header:
        data = "\n".join(
            [header, *(line for line in lines if line.split(",")[0].isdigit())]
        )
        rows = list(csv.DictReader(io.StringIO(data)))
        (root / f"{name}.csv").write_text(data + "\n")
    summaries = {}
    for scenario, operation, visit in sorted(
        {(row["scenario"], row["operation"], row["visit"]) for row in rows}
    ):
        group = [
            row
            for row in rows
            if (row["scenario"], row["operation"], row["visit"])
            == (scenario, operation, visit)
        ]
        metrics = {}
        for key in (
            "preview_ready_ms",
            "elapsed_ms",
            "audio_reset_ms",
            "decoder_close_ms",
            "decoder_open_ms",
            "send_ms",
            "import_ms",
            "gpu_ms",
        ):
            values = sorted(
                float(row[key]) for row in group if row["status"] == "ok" and row[key]
            )
            if values:
                metrics[key] = {
                    "n": len(values),
                    "median": statistics.median(values),
                    "p95": values[math.ceil(len(values) * 0.95) - 1],
                    "max": max(values),
                }
        summaries[f"{scenario}/{operation}/{visit}"] = metrics
    failed = [row for row in rows if row["status"] != "ok"]
    renderer_stats = []
    for line in (root / f"{name}.log").read_text().splitlines():
        if line.startswith("Stats: scope="):
            fields = dict(item.split("=", 1) for item in line.split() if "=" in item)
            if "pipeline_create_ms_n" in fields or "held_submissions_max" in fields:
                renderer_stats.append(fields)
    if "cases" in Path(command[0]).name and result.returncode == 0 and not rows:
        raise RuntimeError(f"{name}: test returned no measurements")
    print(
        f"{name}: exit={result.returncode}, rows={len(rows)}, failures={len(failed)}",
        flush=True,
    )
    status = (
        "timeout"
        if result.returncode == 124
        else "error"
        if result.returncode or failed
        else "ok"
    )
    return {
        "exit": result.returncode,
        "status": status,
        "failures": failed,
        "summary": summaries,
        "renderer_stats": renderer_stats,
    }
