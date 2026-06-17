#!/usr/bin/env python3
"""
Plot simulation CSV data.

Usage:
    python plot_simulation.py <csv_file> [--from DAY] [--to DAY] [--metric METRIC] [--out FILE]
    python plot_simulation.py <csv_file> --style candle [--window N] [--metric METRIC]

Examples:
    python plot_simulation.py simulation_20260421_120000.csv
    python plot_simulation.py simulation_20260421_120000.csv --from 10 --to 20
    python plot_simulation.py simulation_20260421_120000.csv --metric avg_wait_days --style candle
    python plot_simulation.py simulation_20260421_120000.csv --metric patients_resolved --style candle --window 14
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
    "avg_wait_days":      "Avg Wait Days (Resolved)",
    "max_wait_days":      "Max Wait Days (Resolved)",
    "cross_requests_added": "Cross-District Requests Added",
    "cross_resolved":     "Cross-District Resolved",
    "solve_ms":           "Solver Time (ms/day)",
}

# Metrics that make most sense as candlesticks (shown first in help)
CANDLE_GOOD_METRICS = ["patients_resolved", "avg_wait_days", "waitlist_before",
                       "waitlist_after", "satisfaction_rate", "cycles_found"]

# Per-algorithm colors and display names live in plot_style.py so every plot
# uses the same color for a given algorithm. See that module to adjust.
from plot_style import color_for, label_for, marker_for, linestyle_for

# Auto-named plots go here; an explicit --out is still honored verbatim.
PLOTS_DIR = "plots"


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
                "avg_wait_days":     float(row.get("avg_wait_days", 0)),
                "max_wait_days":     int(row.get("max_wait_days", 0)),
                "cross_requests_added": int(row.get("cross_requests_added", 0)),
                "cross_resolved":    int(row.get("cross_resolved", 0)),
                "solve_ms":          float(row.get("solve_ms", 0)),
            }
    return algorithms, data


def _scale(val, is_rate):
    return val * 100 if is_rate else val


def aggregate_candles(alg_data, days_in_range, window, is_rate):
    """Bin days into windows and return OHLC dicts."""
    candles = []
    i = 0
    while i < len(days_in_range):
        chunk = days_in_range[i:i + window]
        vals = [_scale(alg_data[d], is_rate) for d in chunk if d in alg_data]
        if vals:
            candles.append({
                "x":     (chunk[0] + chunk[-1]) / 2,
                "open":  vals[0],
                "close": vals[-1],
                "high":  max(vals),
                "low":   min(vals),
            })
        i += window
    return candles


def draw_candles(ax, candles, color, label, body_width, offset):
    """Draw OHLC candlesticks. Body green when close>=open, hollow otherwise."""
    for c in candles:
        x = c["x"] + offset
        o, cl, hi, lo = c["open"], c["close"], c["high"], c["low"]
        # Wick
        ax.plot([x, x], [lo, hi], color=color, linewidth=1.2, zorder=2)
        # Body
        body_bottom = min(o, cl)
        body_height = abs(cl - o) or (hi - lo) * 0.01  # tiny floor so body visible
        rising = cl >= o
        ax.add_patch(__import__("matplotlib").patches.Rectangle(
            (x - body_width / 2, body_bottom),
            body_width, body_height,
            facecolor=color if rising else "white",
            edgecolor=color,
            linewidth=1.2,
            zorder=3,
        ))
    # Invisible line for legend
    ax.plot([], [], color=color, linewidth=6, alpha=0.7, label=label)


def plot(csv_path, day_from, day_to, metric, out_path, style, window):
    try:
        import matplotlib.pyplot as plt
        import matplotlib.ticker as ticker
    except ImportError:
        sys.exit("matplotlib not installed. Run: pip install matplotlib")

    algorithms, data = load_csv(csv_path)

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

    if style == "candle":
        n = len(algorithms)
        # Body width: fraction of window, shrunk per algorithm count
        body_width = window * 0.55 / max(n, 1)
        # Spread algorithms symmetrically around candle center
        offsets = [(i - (n - 1) / 2) * body_width * 1.15 for i in range(n)]

        for i, alg in enumerate(algorithms):
            color = color_for(alg)
            alg_vals = {d: data[alg][d][metric] for d in days_in_range if d in data[alg]}
            candles = aggregate_candles(alg_vals, days_in_range, window, is_rate)
            draw_candles(ax, candles, color, label_for(alg), body_width, offsets[i])

        # Auto x-limits with padding
        xs = [c["x"] for alg in algorithms
              for c in aggregate_candles(
                  {d: data[alg][d][metric] for d in days_in_range if d in data[alg]},
                  days_in_range, window, is_rate)]
        if xs:
            pad = window * 0.6
            ax.set_xlim(min(xs) - pad, max(xs) + pad)

        subtitle = f"Candlestick ({window}-day windows)"
    else:
        for i, alg in enumerate(algorithms):
            color = color_for(alg)
            xs, ys = [], []
            for d in days_in_range:
                if d in data[alg]:
                    xs.append(d)
                    ys.append(_scale(data[alg][d][metric], is_rate))
            # Space markers out so dense lines stay readable; ~15 markers/line.
            markevery = max(1, len(xs) // 15)
            ax.plot(xs, ys, label=label_for(alg), color=color, linewidth=2,
                    linestyle=linestyle_for(alg), marker=marker_for(alg),
                    markersize=6, markevery=markevery)
        subtitle = "Line"

    ax.set_xlabel("Day", fontsize=13)
    ax.set_ylabel(f"{ylabel}{' (%)' if is_rate else ''}", fontsize=13)
    ax.set_title(
        f"{ylabel} — Days {day_from}–{day_to}  [{subtitle}]",
        fontsize=14,
    )
    ax.legend(fontsize=11)
    ax.grid(True, alpha=0.3)
    ax.xaxis.set_major_locator(ticker.MaxNLocator(integer=True, nbins=20))
    ax.autoscale_view()

    plt.tight_layout()

    if out_path is None:
        ts = datetime.now().strftime("%Y%m%d_%H%M%S")
        os.makedirs(PLOTS_DIR, exist_ok=True)
        out_path = os.path.join(PLOTS_DIR, f"simulation_plot_{metric}_{style}_{day_from}_{day_to}_{ts}.svg")

    plt.savefig(out_path,format="svg")
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
        help=(
            f"Metric to plot (default: patients_resolved). "
            f"Best for candle: {', '.join(CANDLE_GOOD_METRICS)}"
        ),
    )
    parser.add_argument(
        "--style", default="line", choices=["line", "candle"],
        help="Plot style: 'line' (default) or 'candle' (OHLC candlestick per window)",
    )
    parser.add_argument(
        "--window", type=int, default=7, metavar="N",
        help="Window size in days for candle aggregation (default: 7)",
    )
    parser.add_argument("--out", default=None, metavar="FILE",
                        help="Output PNG path (default: auto-named with timestamp)")

    args = parser.parse_args()

    if not os.path.isfile(args.csv):
        sys.exit(f"File not found: {args.csv}")

    if args.style == "candle" and args.window < 2:
        sys.exit("--window must be >= 2 for candle style")

    plot(args.csv, args.day_from, args.day_to, args.metric, args.out, args.style, args.window)


if __name__ == "__main__":
    main()
