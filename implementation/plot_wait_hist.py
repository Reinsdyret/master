#!/usr/bin/env python3
"""
Show and plot wait-time distributions from a simulation wait-histogram CSV.

The Rust simulation writes simulation_wait_hist_<ts>.csv in long format:
    algorithm,kind,wait_days,count
where kind is "resolved" (completed waits) or "outstanding" (patients still on
the waitlist at run end). Because the full per-bucket counts are stored, any
statistic — arbitrary percentiles, alternative starvation thresholds, full CDFs
— can be recomputed here without re-running the simulation.

Usage:
    python plot_wait_hist.py <wait_hist_csv> [--kind resolved|outstanding|both]
        [--style cdf|pdf|both] [--starve DAYS] [--max-days N] [--log] [--out FILE]

Examples:
    python plot_wait_hist.py simulation_wait_hist_20260605_120000.csv
    python plot_wait_hist.py simulation_wait_hist_20260605_120000.csv --style pdf --max-days 200
    python plot_wait_hist.py simulation_wait_hist_20260605_120000.csv --kind both --starve 365
"""

import argparse
import csv
import os
import sys
from collections import defaultdict
from datetime import datetime

# Per-algorithm colors and display names live in plot_style.py so every plot
# uses the same color for a given algorithm. See that module to adjust.
from plot_style import color_for, label_for

# Auto-named plots go here; an explicit --out is still honored verbatim.
PLOTS_DIR = "plots"
PERCENTILES = [50, 90, 95, 99]


def load_csv(path):
    """Return (algorithms, kinds, hist) where hist[(alg, kind)][wait_days] = count."""
    hist = defaultdict(lambda: defaultdict(int))
    algorithms, kinds = [], []
    with open(path, newline="") as f:
        reader = csv.DictReader(f)
        required = {"algorithm", "kind", "wait_days", "count"}
        if not required.issubset(reader.fieldnames or []):
            sys.exit(
                f"CSV missing columns. Need {sorted(required)}, got {reader.fieldnames}.\n"
                "This script expects simulation_wait_hist_<ts>.csv, not the per-day CSV."
            )
        for row in reader:
            alg, kind = row["algorithm"], row["kind"]
            if alg not in algorithms:
                algorithms.append(alg)
            if kind not in kinds:
                kinds.append(kind)
            hist[(alg, kind)][int(row["wait_days"])] += int(row["count"])
    return algorithms, kinds, hist


def hist_stats(buckets, starve_days):
    """Compute summary stats from a {wait_days: count} dict."""
    total = sum(buckets.values())
    if total == 0:
        return None
    items = sorted(buckets.items())
    mean = sum(w * c for w, c in items) / total
    var = sum(c * (w - mean) ** 2 for w, c in items) / total
    std = var ** 0.5
    maxw = max(buckets)

    pct = {}
    for p in PERCENTILES:
        target = max(1, -(-p * total // 100))  # ceil(p/100 * total)
        cum = 0
        pct[p] = maxw
        for w, c in items:
            cum += c
            if cum >= target:
                pct[p] = w
                break

    starved = sum(c for w, c in items if w > starve_days)
    return {
        "count": total, "mean": mean, "std": std, "max": maxw,
        "pct": pct, "starved": starved,
        "starved_frac": starved / total,
    }


def print_stats(algorithms, kinds, hist, kinds_to_show, starve_days):
    print(f"\nWait-time distribution stats  (starvation threshold = {starve_days}d)")
    hdr = (
        f"{'Algorithm':<18}  {'Kind':<11}  {'N':>8}  {'Mean':>7}  {'Std':>7}  "
        + "  ".join(f"P{p:<3}".rjust(6) for p in PERCENTILES)
        + f"  {'Max':>6}  {'Starved':>9}"
    )
    print(hdr)
    print("-" * len(hdr))
    for alg in algorithms:
        for kind in kinds:
            if kind not in kinds_to_show:
                continue
            s = hist_stats(hist[(alg, kind)], starve_days)
            if s is None:
                continue
            pcts = "  ".join(f"{s['pct'][p]:>6}" for p in PERCENTILES)
            print(
                f"{alg:<18}  {kind:<11}  {s['count']:>8}  {s['mean']:>7.1f}  {s['std']:>7.1f}  "
                f"{pcts}  {s['max']:>6}  {s['starved']:>6} ({s['starved_frac']*100:>3.0f}%)"
            )
    print("-" * len(hdr))


def to_series(buckets, max_days):
    """Dense (xs, counts) arrays from 0..=max_days for a bucket dict."""
    hi = max_days if max_days is not None else (max(buckets) if buckets else 0)
    xs = list(range(hi + 1))
    counts = [buckets.get(x, 0) for x in xs]
    # Fold the tail beyond max_days into the last visible bucket so mass is preserved.
    if max_days is not None:
        tail = sum(c for w, c in buckets.items() if w > max_days)
        if counts:
            counts[-1] += tail
    return xs, counts


def plot(algorithms, kinds, hist, kinds_to_show, style, max_days, log, out_path, csv_path):
    try:
        import matplotlib.pyplot as plt
    except ImportError:
        sys.exit("matplotlib not installed. Run: pip install matplotlib")

    panels = ["cdf", "pdf"] if style == "both" else [style]
    fig, axes = plt.subplots(1, len(panels), figsize=(8 * len(panels), 6), squeeze=False)
    axes = axes[0]

    linestyles = {"resolved": "-", "outstanding": "--"}

    for ax, panel in zip(axes, panels):
        for i, alg in enumerate(algorithms):
            color = color_for(alg)
            for kind in kinds:
                if kind not in kinds_to_show:
                    continue
                buckets = hist[(alg, kind)]
                if not buckets:
                    continue
                xs, counts = to_series(buckets, max_days)
                total = sum(counts) or 1
                label = label_for(alg) if kind == "resolved" else f"{label_for(alg)} ({kind})"
                if panel == "cdf":
                    cum, ys = 0, []
                    for c in counts:
                        cum += c
                        ys.append(cum / total)
                else:  # pdf — fraction of mass per wait-day
                    ys = [c / total for c in counts]
                ax.plot(
                    xs, ys, color=color,
                    linestyle=linestyles.get(kind, "-"),
                    linewidth=2, label=label,
                )

        ax.set_xlabel("Wait time (days)", fontsize=12)
        ax.set_ylabel("Cumulative fraction" if panel == "cdf" else "Fraction of requests", fontsize=12)
        ax.set_title(("CDF" if panel == "cdf" else "Distribution") + " of wait times", fontsize=13)
        if log:
            ax.set_yscale("log")
        ax.grid(True, alpha=0.3)
        ax.legend(fontsize=9)

    fig.suptitle(os.path.basename(csv_path), fontsize=12)
    plt.tight_layout()

    if out_path is None:
        ts = datetime.now().strftime("%Y%m%d_%H%M%S")
        os.makedirs(PLOTS_DIR, exist_ok=True)
        out_path = os.path.join(PLOTS_DIR, f"wait_hist_{style}_{ts}.png")
    plt.savefig(out_path, dpi=150)
    print(f"\nPlot saved to: {out_path}")


def main():
    parser = argparse.ArgumentParser(description="Show/plot wait-time distributions.")
    parser.add_argument("csv", help="Path to simulation_wait_hist_<ts>.csv")
    parser.add_argument("--kind", default="resolved",
                        choices=["resolved", "outstanding", "both"],
                        help="Which wait kind to show (default: resolved)")
    parser.add_argument("--style", default="cdf", choices=["cdf", "pdf", "both"],
                        help="Plot style (default: cdf)")
    parser.add_argument("--starve", type=int, default=90, metavar="DAYS",
                        help="Starvation threshold for the printed stats (default: 90)")
    parser.add_argument("--max-days", type=int, default=None, metavar="N",
                        help="Clamp x-axis to N days; mass beyond is folded into the last bucket")
    parser.add_argument("--log", action="store_true", help="Log-scale the y-axis")
    parser.add_argument("--no-plot", action="store_true", help="Print stats only, skip the plot")
    parser.add_argument("--out", default=None, metavar="FILE", help="Output PNG path")
    args = parser.parse_args()

    if not os.path.isfile(args.csv):
        sys.exit(f"File not found: {args.csv}")

    algorithms, kinds, hist = load_csv(args.csv)
    kinds_to_show = kinds if args.kind == "both" else [args.kind]

    print_stats(algorithms, kinds, hist, kinds_to_show, args.starve)

    if not args.no_plot:
        plot(algorithms, kinds, hist, kinds_to_show, args.style,
             args.max_days, args.log, args.out, args.csv)


if __name__ == "__main__":
    main()
