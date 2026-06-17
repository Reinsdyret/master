#!/usr/bin/env python3
"""
Bar-chart comparison of algorithms from a simulation summary CSV.

The Rust simulation writes simulation_summary_<ts>.csv with one row per
algorithm and every aggregate figure as a column. This script draws a grouped
bar dashboard (one panel per metric, algorithms on the x-axis) and prints the
same numbers as a table.

Usage:
    python plot_summary.py <summary_csv> [--metrics m1,m2,...] [--out FILE] [--no-plot]

Examples:
    python plot_summary.py simulation_summary_20260605_120000.csv
    python plot_summary.py simulation_summary_20260605_120000.csv \
        --metrics resolved_avg_wait,resolved_p99,overall_max_wait
"""

import argparse
import csv
import os
import sys
from datetime import datetime

# Per-algorithm colors and display names live in plot_style.py so every plot
# uses the same color for a given algorithm. See that module to adjust.
from plot_style import color_for, label_for

# Auto-named plots go here; an explicit --out is still honored verbatim.
PLOTS_DIR = "plots"

# label, is_percent (value stored as 0..1 -> shown as %)
METRICS = {
    "total_resolved":            ("Total resolved", False),
    "avg_daily_satisfaction_rate": ("Avg satisfaction", True),
    "avg_waitlist_size":         ("Avg waitlist size", False),
    "max_waitlist_size":         ("Max waitlist size", False),
    "final_waitlist_size":       ("Final waitlist size", False),
    "avg_cycles_per_day":        ("Avg cycles/day", False),
    "avg_cycle_length_overall":  ("Avg cycle length", False),
    "resolved_avg_wait":         ("Resolved avg wait (d)", False),
    "resolved_std_wait":         ("Resolved std wait (d)", False),
    "resolved_p50":              ("Resolved P50 wait (d)", False),
    "resolved_p90":              ("Resolved P90 wait (d)", False),
    "resolved_p95":              ("Resolved P95 wait (d)", False),
    "resolved_p99":              ("Resolved P99 wait (d)", False),
    "resolved_max":              ("Resolved max wait (d)", False),
    "outstanding_count":         ("Outstanding (never resolved)", False),
    "outstanding_avg_wait":      ("Outstanding avg wait (d)", False),
    "outstanding_max":           ("Outstanding max wait (d)", False),
    "overall_avg_wait":          ("Overall avg wait (d)", False),
    "overall_max_wait":          ("Overall max wait (d)", False),
    "starved_resolved":          ("Starved resolved", False),
    "starved_outstanding":       ("Starved outstanding", False),
    "total_solve_ms":            ("Total solve time (s)", False),
    "avg_solve_ms":              ("Avg solve time/day (ms)", False),
    "max_solve_ms":              ("Max solve time/day (ms)", False),
}

DEFAULT_METRICS = [
    "total_resolved",
    "avg_daily_satisfaction_rate",
    "resolved_avg_wait",
    "resolved_p99",
    "overall_max_wait",
    "outstanding_count",
]


def load_csv(path):
    with open(path, newline="") as f:
        reader = csv.DictReader(f)
        if "algorithm" not in (reader.fieldnames or []):
            sys.exit(
                "CSV has no 'algorithm' column.\n"
                "This script expects simulation_summary_<ts>.csv (one row per algorithm)."
            )
        rows = list(reader)
    if not rows:
        sys.exit("Summary CSV is empty.")
    return rows


# Metrics stored in ms but displayed in seconds (value divided by 1000).
MS_TO_S = {"total_solve_ms"}


def fval(row, metric):
    try:
        val = float(row[metric])
    except (KeyError, ValueError):
        return 0.0
    if metric in MS_TO_S:
        val /= 1000.0
    return val


def fmt(val, is_percent):
    if is_percent:
        return f"{val * 100:.1f}%"
    if abs(val - round(val)) < 1e-9:
        return f"{int(round(val))}"
    return f"{val:.2f}"


def print_table(rows, metrics):
    algs = [label_for(r["algorithm"]) for r in rows]
    width = max(len(a) for a in algs + ["Algorithm"])
    cols = [METRICS.get(m, (m, False))[0] for m in metrics]
    colw = [max(len(c), 10) for c in cols]

    header = f"{'Algorithm':<{width}}  " + "  ".join(f"{c:>{w}}" for c, w in zip(cols, colw))
    print("\n" + header)
    print("-" * len(header))
    for r in rows:
        cells = []
        for m, w in zip(metrics, colw):
            _, is_pct = METRICS.get(m, (m, False))
            cells.append(f"{fmt(fval(r, m), is_pct):>{w}}")
        print(f"{label_for(r['algorithm']):<{width}}  " + "  ".join(cells))
    print("-" * len(header))


def plot(rows, metrics, out_path, csv_path):
    try:
        import matplotlib.pyplot as plt
    except ImportError:
        sys.exit("matplotlib not installed. Run: pip install matplotlib")

    algs = [r["algorithm"] for r in rows]
    n = len(metrics)
    ncols = min(3, n)
    nrows = -(-n // ncols)  # ceil
    fig, axes = plt.subplots(nrows, ncols, figsize=(6 * ncols, 4.2 * nrows), squeeze=False)

    for idx, metric in enumerate(metrics):
        ax = axes[idx // ncols][idx % ncols]
        label, is_pct = METRICS.get(metric, (metric, False))
        vals = [fval(r, metric) * (100 if is_pct else 1) for r in rows]
        bars = ax.bar(range(len(algs)), vals,
                      color=[color_for(a) for a in algs])
        ax.set_title(label, fontsize=12)
        ax.set_xticks(range(len(algs)))
        ax.set_xticklabels([label_for(a) for a in algs], rotation=30, ha="right", fontsize=9)
        ax.grid(True, axis="y", alpha=0.3)
        for b, v in zip(bars, vals):
            ax.annotate(f"{v:.1f}" if is_pct else fmt(v, False),
                        (b.get_x() + b.get_width() / 2, v),
                        ha="center", va="bottom", fontsize=8,
                        xytext=(0, 2), textcoords="offset points")
        vmax = max(vals) if vals else 0
        if vmax > 0:
            ax.set_ylim(top=vmax * 1.12)

    # Hide unused panels
    for j in range(n, nrows * ncols):
        axes[j // ncols][j % ncols].axis("off")

    metric_labels = [METRICS.get(m, (m, False))[0] for m in metrics]
    fig.suptitle("Algorithm comparison — " + ", ".join(metric_labels), fontsize=13)
    plt.tight_layout()

    if out_path is None:
        ts = datetime.now().strftime("%Y%m%d_%H%M%S")
        os.makedirs(PLOTS_DIR, exist_ok=True)
        out_path = os.path.join(PLOTS_DIR, f"summary_bars_{ts}.svg")
    plt.savefig(out_path, format="svg")
    print(f"\nPlot saved to: {out_path}")


def main():
    parser = argparse.ArgumentParser(description="Bar-chart comparison from a summary CSV.")
    parser.add_argument("csv", help="Path to simulation_summary_<ts>.csv")
    parser.add_argument("--metrics", default=None,
                        help="Comma-separated metric columns (default: a 6-metric dashboard). "
                             "Available: " + ", ".join(METRICS.keys()))
    parser.add_argument("--no-plot", action="store_true", help="Print table only, skip the plot")
    parser.add_argument("--out", default=None, metavar="FILE", help="Output PNG path")
    args = parser.parse_args()

    if not os.path.isfile(args.csv):
        sys.exit(f"File not found: {args.csv}")

    if args.metrics:
        metrics = [m.strip() for m in args.metrics.split(",") if m.strip()]
        unknown = [m for m in metrics if m not in METRICS]
        if unknown:
            print(f"Warning: unrecognized metrics plotted as-is: {unknown}", file=sys.stderr)
    else:
        metrics = DEFAULT_METRICS

    rows = load_csv(args.csv)
    print_table(rows, metrics)
    if not args.no_plot:
        plot(rows, metrics, args.out, args.csv)


if __name__ == "__main__":
    main()
