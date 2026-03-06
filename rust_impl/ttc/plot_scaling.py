import csv
import os
import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np

def load_csv(path):
    rows = []
    with open(path) as f:
        for row in csv.DictReader(f):
            row["num_patients"] = int(row["num_patients"])
            row["patients_satisfied"] = int(row["patients_satisfied"])
            row["patients_wanting_switch"] = int(row["patients_wanting_switch"])
            row["satisfaction_rate"] = float(row["satisfaction_rate"])
            row["time_ms"] = int(row["time_ms"])
            rows.append(row)
    return rows

rows = load_csv("benchmark_scaling.csv")
if os.path.exists("benchmark_scaling_sa.csv"):
    rows += load_csv("benchmark_scaling_sa.csv")

algos   = ["CyclePacker", "TTC_StrictPriority", "SA_Heuristic"]
labels  = {"CyclePacker": "CyclePacker (exact)", "TTC_StrictPriority": "TTC (StrictPriority)", "SA_Heuristic": "SA Heuristic"}
colors  = {"CyclePacker": "#e05c5c", "TTC_StrictPriority": "#4a90d9", "SA_Heuristic": "#4caf73"}
markers = {"CyclePacker": "o", "TTC_StrictPriority": "s", "SA_Heuristic": "^"}

# Only keep algos that have data
algos = [a for a in algos if any(r["algorithm"] == a for r in rows)]

def get(algo, key):
    return [(r["num_patients"], r[key]) for r in rows if r["algorithm"] == algo]

fig, axes = plt.subplots(1, 2, figsize=(14, 5))
fig.suptitle("CyclePacker vs TTC vs SA Heuristic — Scaling Comparison", fontsize=13, fontweight="bold")

# --- Plot 1: Runtime (log-log) ---
ax = axes[0]
for algo in algos:
    pts = get(algo, "time_ms")
    xs = [p[0] for p in pts]
    ys = [max(p[1], 1) for p in pts]   # clamp 0ms → 1ms so log scale works
    ax.plot(xs, ys, marker=markers[algo], color=colors[algo], label=labels[algo],
            linewidth=2, markersize=7)

ax.set_xscale("log")
ax.set_yscale("log")
ax.set_xlabel("Number of patients (log scale)")
ax.set_ylabel("Runtime (ms, log scale)")
ax.set_title("Runtime Scaling")
ax.legend()
ax.grid(True, which="major", linestyle="--", alpha=0.4)
ax.xaxis.set_major_formatter(ticker.FuncFormatter(lambda x, _: f"{int(x):,}"))
ax.yaxis.set_major_formatter(ticker.FuncFormatter(lambda x, _: f"{int(x):,}"))

# --- Plot 2: Satisfaction rate (grouped bars) ---
ax = axes[1]
ref_pts = get(algos[0], "satisfaction_rate")
n_groups = len(ref_pts)
x = np.arange(n_groups)
width = 0.8 / len(algos)

for i, algo in enumerate(algos):
    pts = get(algo, "satisfaction_rate")
    rates = [p[1] * 100 for p in pts]
    ax.bar(x + i * width, rates, width, label=labels[algo], color=colors[algo], alpha=0.85)

xlabels = [f"{p[0]:,}" for p in ref_pts]
ax.set_xticks(x + width * (len(algos) - 1) / 2)
ax.set_xticklabels(xlabels, rotation=15)
ax.set_xlabel("Number of patients")
ax.set_ylabel("Satisfaction rate (%)")
ax.set_title("Solution Quality (Satisfaction Rate)")
ax.set_ylim(0, 100)
ax.legend()
ax.grid(True, axis="y", linestyle="--", alpha=0.4)

plt.tight_layout()
out = "benchmark_scaling.png"
plt.savefig(out, dpi=150)
print(f"Saved {out}")
plt.show()
