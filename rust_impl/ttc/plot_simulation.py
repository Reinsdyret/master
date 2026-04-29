#!/usr/bin/env python3
"""
Plot simulation CSV data.

Usage:
    python plot_simulation.py <csv_file> [--from DAY] [--to DAY] [--metric METRIC] [--out FILE]

Examples:
    python plot_simulation.py simulation_20260421_120000.csv
    python plot_simulation.py simulation_20260421_120000.csv --from 10 --to 20
    python plot_simulation.py simulation_20260421_120000.csv --from 100 --to 360
    python plot_simulation.py simulation_20260421_120000.csv --metric satisfaction_rate
"""

import argparse
import csv
import os
import sys
from collections import defaultdict
from datetime import datetime

METRICS = {
    "patients_resolved":  "Patients Resolved",
    "waitlist_before":    "Waitlist Size (Before)",
    "waitlist_after":     "Waitlist Size (After)",
    "satisfaction_rate":  "Satisfaction Rate",
    "cycles_found":       "Cycles Found",
    "avg_cycle_length":   "Avg Cycle Length",
    "new_requests":       "New Requests Added",
}

COLORS = ["#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd", "#8c564b"]


def load_csv(path):
    data = defaultdict(lambda: defaultdict(dict))
    algorithms = []
    with open(path, newline="") as f:
        reader = csv.DictReader(f)
        for row in reader:
            alg = row["algorithm"]
            day = int(row["day"])
            if alg not in algorithms:
                algorithms.append(alg)
            data[alg][day] = {
                "waitlist_before":   int(row["waitlist_before"]),
                "patients_resolved": int(row["patients_resolved"]),
                "new_requests":      int(row["new_requests"]),
                "waitlist_after":    int(row["waitlist_after"]),
                "satisfaction_rate": float(row["satisfaction_rate"]),
                "cycles_found":      int(row["cycles_found"]),
                "avg_cycle_length":  float(row["avg_cycle_length"]),
                "max_cycle_length":  int(row["max_cycle_length"]),
            }
    return algorithms, data


def plot(csv_path, day_from, day_to, metric, out_path):
    try:
        import matplotlib.pyplot as plt
        import matplotlib.ticker as ticker
    except ImportError:
        sys.exit("matplotlib not installed. Run: pip install matplotlib")

    algorithms, data = load_csv(csv_path)

    # Determine day range
    all_days = sorted({d for alg in algorithms for d in data[alg]})
    if not all_days:
        sys.exit("No data found in CSV.")

    if day_from is None:
        day_from = all_days[0]
    if day_to is None:
        day_to = all_days[-1]

    if day_from > day_to:
        sys.exit(f"--from {day_from} > --to {day_to}")

    days_in_range = [d for d in all_days if day_from <= d <= day_to]
    if not days_in_range:
        sys.exit(f"No days in range [{day_from}, {day_to}].")

    ylabel = METRICS.get(metric, metric)
    is_rate = metric == "satisfaction_rate"

    fig, ax = plt.subplots(figsize=(14, 7))

    for i, alg in enumerate(algorithms):
        color = COLORS[i % len(COLORS)]
        xs = []
        ys = []
        for d in days_in_range:
            if d in data[alg]:
                xs.append(d)
                val = data[alg][d][metric]
                ys.append(val * 100 if is_rate else val)
        ax.plot(xs, ys, label=alg, color=color, linewidth=2, marker="o", markersize=3)

    ax.set_xlabel("Day", fontsize=13)
    ax.set_ylabel(f"{ylabel}{' (%)' if is_rate else ''}", fontsize=13)
    ax.set_title(
        f"{ylabel} — Days {day_from}–{day_to}\n{os.path.basename(csv_path)}",
        fontsize=14,
    )
    ax.legend(fontsize=11)
    ax.grid(True, alpha=0.3)
    ax.xaxis.set_major_locator(ticker.MaxNLocator(integer=True, nbins=20))

    plt.tight_layout()

    if out_path is None:
        ts = datetime.now().strftime("%Y%m%d_%H%M%S")
        out_path = f"simulation_plot_{metric}_{day_from}_{day_to}_{ts}.png"

    plt.savefig(out_path, dpi=150)
    print(f"Plot saved to: {out_path}")


def main():
    parser = argparse.ArgumentParser(description="Plot simulation CSV data.")
    parser.add_argument("csv", help="Path to simulation CSV file")
    parser.add_argument("--from", dest="day_from", type=int, default=None,
                        metavar="DAY", help="First day to include (default: first in data)")
    parser.add_argument("--to", dest="day_to", type=int, default=None,
                        metavar="DAY", help="Last day to include (default: last in data)")
    parser.add_argument(
        "--metric", default="patients_resolved",
        choices=list(METRICS.keys()),
        help=f"Column to plot (default: patients_resolved). Options: {', '.join(METRICS)}",
    )
    parser.add_argument("--out", default=None, metavar="FILE",
                        help="Output PNG path (default: auto-named with timestamp)")

    args = parser.parse_args()

    if not os.path.isfile(args.csv):
        sys.exit(f"File not found: {args.csv}")

    plot(args.csv, args.day_from, args.day_to, args.metric, args.out)


if __name__ == "__main__":
    main()
