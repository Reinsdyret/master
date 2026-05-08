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

# Discover all algorithms present in the data
all_algos = list(dict.fromkeys(r["algorithm"] for r in rows))

labels = {
    "CyclePacker":                "CyclePacker (exact)",
    "CyclePacker_PriorityWeighted": "CyclePacker (priority-weighted)",
    "TTC_StrictPriority":         "Greedy DFS (strict priority)",
    "Huitfeldt_TTC":              "Huitfeldt TTC",
}
colors = {
    "CyclePacker":                "#e05c5c",
    "CyclePacker_PriorityWeighted": "#e0964a",
    "TTC_StrictPriority":         "#4a90d9",
    "Huitfeldt_TTC":              "#4caf73",
}
markers = {
    "CyclePacker":                "o",
    "CyclePacker_PriorityWeighted": "D",
    "TTC_StrictPriority":         "s",
    "Huitfeldt_TTC":              "^",
}

# Fallback style for any algo not in the dicts above
_fallback_colors  = ["#9b59b6", "#f1c40f", "#1abc9c", "#e74c3c"]
_fallback_markers = ["v", "P", "X", "*"]
for i, algo in enumerate([a for a in all_algos if a not in colors]):
    colors[algo]  = _fallback_colors[i % len(_fallback_colors)]
    markers[algo] = _fallback_markers[i % len(_fallback_markers)]
    labels.setdefault(algo, algo)

def get(algo, key):
    pts = [(r["num_patients"], r[key]) for r in rows if r["algorithm"] == algo]
    return sorted(pts)

fig, axes = plt.subplots(1, 2, figsize=(14, 5))
fig.suptitle("Algorithm Comparison — Scaling", fontsize=13, fontweight="bold")

# --- Plot 1: Runtime (log-log) ---
ax = axes[0]
for algo in all_algos:
    pts = get(algo, "time_ms")
    xs = [p[0] for p in pts]
    ys = [max(p[1], 1) for p in pts]  # clamp 0ms → 1ms for log scale
    ax.plot(xs, ys, marker=markers[algo], color=colors[algo],
            label=labels[algo], linewidth=2, markersize=7)

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
ref_pts = get(all_algos[0], "satisfaction_rate")
n_groups = len(ref_pts)
x = np.arange(n_groups)
width = 0.8 / len(all_algos)

for i, algo in enumerate(all_algos):
    pts = get(algo, "satisfaction_rate")
    rates = [p[1] * 100 for p in pts]
    ax.bar(x + i * width, rates, width,
           label=labels[algo], color=colors[algo], alpha=0.85)

xlabels = [f"{p[0]:,}" for p in ref_pts]
ax.set_xticks(x + width * (len(all_algos) - 1) / 2)
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
